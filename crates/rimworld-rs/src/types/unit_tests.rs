//! Theme: unit_tests. Split from `types.rs`.
#![allow(unused_imports, unused_macros)] // the imports and the macros are the module's shared vocabulary, used across the theme files

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use super::*;
use crate::{host, Result};

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn patch_call_parses_a_full_payload() {
        let payload = json!({
            "handler": "on_damage",
            "kind": "postfix",
            "target": "Verse.Pawn.Kill(DamageInfo)",
            "instance": { "$ref": 0, "$type": "Verse.Pawn", "$str": "Alice" },
            "args": [ 7, "text", true, { "$ref": 1, "$type": "Verse.Thing" } ],
            "result": 3.5
        });

        let call = PatchCall::from_value(&payload);
        assert_eq!(call.handler, "on_damage");
        assert_eq!(call.kind, PatchKind::Postfix);
        assert_eq!(call.target, "Verse.Pawn.Kill(DamageInfo)");
        assert_eq!(call.instance_obj().map(|handle| handle.id), Some(0));
        assert_eq!(call.arg_i64(0), Some(7));
        assert_eq!(call.arg_str(1), Some("text"));
        assert_eq!(call.arg_bool(2), Some(true));
        assert_eq!(call.arg_obj(3).map(|handle| handle.type_name), Some("Verse.Thing".to_string()));
        assert_eq!(call.result_f64(), Some(3.5));
    }

    #[test]
    fn patch_call_tolerates_missing_fields() {
        let call = PatchCall::from_value(&json!({ "handler": "x" }));
        assert_eq!(call.handler, "x");
        assert_eq!(call.kind, PatchKind::Prefix);
        assert!(call.args.is_empty());
        assert!(call.instance.is_none());
        assert!(call.result.is_none());
        assert_eq!(call.arg_i64(4), None);
    }

    #[test]
    fn patch_outcome_serializes_every_change() {
        let outcome = PatchOutcome::default().set_arg(0, 25).set_arg(2, "text").with_result(true).skip_original();

        let value: Value = serde_json::from_str(&outcome.to_json()).expect("valid json");
        assert_eq!(value["ok"], Value::Bool(true));
        assert_eq!(value["skipOriginal"], Value::Bool(true));
        assert_eq!(value["result"], Value::Bool(true));
        assert_eq!(value["args"]["0"], json!(25));
        assert_eq!(value["args"]["2"], json!("text"));
    }

    #[test]
    fn patch_outcome_default_is_a_no_op() {
        let value: Value = serde_json::from_str(&PatchOutcome::default().to_json()).expect("valid json");
        assert_eq!(value["skipOriginal"], Value::Bool(false));
        assert!(value.get("result").is_none());
        assert!(value.get("args").is_none());
    }

    #[test]
    fn object_references_are_recognized() {
        let handle = ObjRef::from_value(&json!({ "$ref": 12, "$type": "Verse.Map" })).expect("handle");
        assert_eq!(handle.id, 12);
        assert_eq!(handle.type_name, "Verse.Map");
        assert!(handle.text.is_none());

        assert!(ObjRef::from_value(&json!(5)).is_none());
        assert!(ObjRef::from_value(&json!({ "count": 1 })).is_none());
    }

    /// A deterministic generator: `rand`'s `StdRng` behind the one call this module needs. The seed is fixed, so
    /// a failing round stays reproducible - and `proptest` is a dev-dependency already, so it carries `rand`
    /// anyway: the hand-written xorshift only added a second generator to keep working.
    struct Rng(rand::rngs::StdRng);

    impl Rng {
        fn seeded(seed: u64) -> Self {
            Self(rand::SeedableRng::seed_from_u64(seed))
        }

        fn below(&mut self, limit: u64) -> u64 {
            rand::RngExt::random_range(&mut self.0, 0..limit)
        }
    }

    fn random_value(rng: &mut Rng, depth: u32) -> Value {
        match rng.below(if depth >= 3 { 6 } else { 8 }) {
            0 => Value::Null,
            1 => Value::Bool(rng.below(2) == 0),
            2 => Value::from(rng.below(1_000) as i64 - 500),
            3 => Value::from(rng.below(1_000_000) as f64 / 997.0),
            4 => Value::from(format!("text{}", rng.below(100))),
            5 => json!({ "$ref": rng.below(50) as i64, "$type": "Verse.Thing", "$str": "thing" }),
            6 => Value::Array((0..rng.below(4)).map(|_| random_value(rng, depth + 1)).collect()),
            _ => {
                let mut object = Map::new();
                object.insert("handler".to_string(), Value::from(format!("h{}", rng.below(10))));
                object.insert("args".to_string(), Value::Array(vec![random_value(rng, depth + 1)]));
                Value::Object(object)
            }
        }
    }

    #[test]
    fn caravan_payload_matches_the_bridge() {
        // The exact shape bridge/src/WorldOps.cs sends: the caravan check in the game caught this
        // field being an int once, which no offline check could see (the mock refuses caravans).
        let payload = serde_json::json!({
            "label": "Caravan 1",
            "tile": 42,
            "pawns": [
                { "label": "Wildfire", "handle": { "$ref": 3 } },
                { "label": "Ash", "handle": { "$ref": 4 } },
            ],
            "handle": { "$ref": 5 },
        });

        let caravan: CaravanInfo = serde_json::from_value(payload).expect("the caravan payload parses");
        assert_eq!(caravan.label.as_deref(), Some("Caravan 1"));
        assert_eq!(caravan.tile, 42);
        assert_eq!(caravan.pawns.len(), 2);
        assert_eq!(caravan.pawns[0].label.as_deref(), Some("Wildfire"));
        assert!(caravan.pawns[0].handle.is_some());
        assert!(caravan.handle.is_some());
    }

    #[test]
    fn random_payloads_never_break_the_lenient_parsers() {
        let mut rng = Rng::seeded(0x2026_1014_CAFE_BABE);
        for round in 0..500 {
            let value = random_value(&mut rng, 0);

            let call = PatchCall::from_value(&value);
            let redirect = RedirectCall::from_value(&value);
            let _ = ObjRef::from_value(&value);

            // Whatever came in, an outcome built from it must still serialize to valid JSON.
            let mut outcome = PatchOutcome::default();
            if let Some(first) = call.args.first() {
                outcome = outcome.set_arg(0, first.clone());
            }
            if let Some(result) = call.result_i64() {
                outcome = outcome.with_result(result);
            }

            let encoded = outcome.to_json();
            let decoded: Value =
                serde_json::from_str(&encoded).unwrap_or_else(|error| panic!("round {round}: {error} in {encoded}"));

            assert_eq!(decoded["ok"], Value::Bool(true), "round {round}");
            assert!(redirect.args.len() <= 4, "round {round}");
        }
    }

    #[test]
    fn object_references_round_trip() {
        let mut rng = Rng::seeded(0x5EED_1234_5678_9ABC);
        for _ in 0..200 {
            let handle = ObjRef {
                id: rng.below(10_000) as i32,
                type_name: format!("Verse.Type{}", rng.below(20)),
                text: if rng.below(2) == 0 { None } else { Some(format!("label{}", rng.below(50))) },
            };

            let parsed = ObjRef::from_value(&handle.to_value()).expect("a handle round-trips");
            assert_eq!(parsed, handle);
        }
    }

    #[test]
    fn game_info_reports_the_harness() {
        let real: GameInfo = serde_json::from_str("{\"hasGame\":true,\"tick\":10}").expect("parses");
        assert!(!real.is_mock());

        let mock: GameInfo =
            serde_json::from_str("{\"hasGame\":true,\"tick\":10,\"harness\":\"mock\"}").expect("parses");
        assert!(mock.is_mock());
    }

    /// Property tests on top of the deterministic cases above: the protocol is lenient by design, so the
    /// questions worth asking are "for *every* shape, does it still hold?" - and proptest shrinks a failure to
    /// the smallest payload that breaks it, which the deterministic generator above cannot do.
    mod properties {
        use proptest::prelude::*;

        use super::*;

        /// An arbitrary JSON value: leaves of every kind the bridge writes, nested to a depth, and free to be
        /// nonsense - which is exactly what a lenient parser has to survive.
        fn any_json() -> impl Strategy<Value = Value> {
            let leaf = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::from),
                any::<i64>().prop_map(Value::from),
                any::<f64>().prop_map(Value::from),
                ".*".prop_map(Value::from),
                (0i32..50, "[A-Za-z\\.]{0,24}", ".*")
                    .prop_map(|(id, type_name, text)| { json!({ "$ref": id, "$type": type_name, "$str": text }) }),
                ("[a-zA-Z_]{1,8}", "[A-Za-z\\.]{0,24}").prop_map(|(type_name, member)| {
                    json!({ "op": "ref_get", "type": type_name, "member": member })
                }),
            ];

            leaf.prop_recursive(4, 32, 4, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                    prop::collection::hash_map("[A-Za-z_]{1,8}", inner, 0..4)
                        .prop_map(|map| Value::Object(map.into_iter().collect())),
                ]
            })
        }

        proptest! {
            /// Whatever arrives from the bridge, the parsers must not panic and an outcome built from what they
            /// read must serialize back to valid JSON.
            #[test]
            fn arbitrary_payloads_never_break_the_lenient_parsers(value in any_json()) {
                let call = PatchCall::from_value(&value);
                let redirect = RedirectCall::from_value(&value);
                let _ = ObjRef::from_value(&value);

                let mut outcome = PatchOutcome::default();
                if let Some(first) = call.args.first() {
                    outcome = outcome.set_arg(0, first.clone());
                }
                if let Some(result) = call.result_i64() {
                    outcome = outcome.with_result(result);
                }

                let encoded = outcome.to_json();
                let decoded: Value = serde_json::from_str(&encoded).expect("the outcome reserializes");
                prop_assert_eq!(decoded["ok"].clone(), Value::Bool(true));
                prop_assert!(redirect.args.len() <= 4);
            }

            /// A handle survives the round trip through the shape it travels in, whatever it holds.
            #[test]
            fn object_references_round_trip_through_the_wire(
                id in 0i32..1_000_000,
                type_name in "[A-Za-z\\.]{0,40}",
                text in proptest::option::of(".*"),
            ) {
                let handle = ObjRef { id, type_name, text };
                let parsed = ObjRef::from_value(&handle.to_value()).expect("a handle round-trips");
                prop_assert_eq!(parsed, handle);
            }

            /// Every argument an outcome carries comes back as the same JSON - for a value that **came from
            /// JSON**, which is the only kind the wire hands the SDK.
            ///
            /// This started as `value in any_json()` and proptest found `[-8.419636492640652e+187]`: it comes
            /// back as `...653e+187`, one ULP away, because `serde_json` writes an `f64` in its shortest form
            /// and reading that text back need not land on the same double. A `Value` built in Rust from a raw
            /// `f64` can shift that way; one parsed from JSON text cannot - and the bridge always writes text.
            #[test]
            fn outcome_arguments_survive_serialization(
                index in 0usize..8,
                text in ".{0,120}",
            ) {
                let Ok(value) = serde_json::from_str::<Value>(&text) else {
                    return Ok(());
                };

                if value.is_null() {
                    return Ok(());
                }

                let encoded = PatchOutcome::default().set_arg(index, value.clone()).to_json();
                let decoded: Value = serde_json::from_str(&encoded).expect("valid json");
                let key = index.to_string();
                prop_assert_eq!(&decoded["args"][key.as_str()], &value);
            }
        }
    }
}
