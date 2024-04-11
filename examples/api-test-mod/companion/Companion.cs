using System;
using System.Collections.Generic;
using RimWorld;
using UnityEngine;
using Verse;
using RustBridge;

namespace ApiTestCompanion
{
    /// <summary>
    /// The managed half of the self-test mod: proves that a Rust mod can own a companion C#
    /// assembly, call into it from Rust and receive events back from it.
    /// </summary>
    public static class Bridge
    {
        /// <summary>Called from Rust with `companion_call`; the answer is marshaled back.</summary>
        public static string Describe(string what)
        {
            return "companion saw '" + what + "'";
        }

        /// <summary>An overload pair, so the self-test can prove overload selection by paramTypes.</summary>
        public static string Describe(string what, int repeat)
        {
            return "companion saw '" + what + "' " + repeat + " times";
        }

        /// <summary>A value-type return, marshaled back as a number.</summary>
        public static int Add(int left, int right)
        {
            return left + right;
        }

        /// <summary>Asks the Rust side something and returns its answer.</summary>
        public static string AskRust(string question)
        {
            return Companion.Request("ApiTestMod", "companion_question", new Dictionary<string, object>
            {
                ["question"] = question,
            });
        }

        /// <summary>The destination of the redirected call: `harmony_redirect` points at this one.</summary>
        public static string DescribeRedirected(string what)
        {
            return "redirected '" + what + "'";
        }

        /// <summary>Calls <see cref="Describe"/>; a transpiler can point that call somewhere else.</summary>
        public static string DescribeThroughRedirect()
        {
            return Describe("redirect-me");
        }

        /// <summary>Calls <see cref="Add"/>; a transpiler can send that call into Rust instead.</summary>
        public static int AddThroughReplace()
        {
            return Add(2, 40);
        }
    }

    /// <summary>
    /// Reports its thing to Rust when it spawns and once per life when it ticks. Attached to Steel
    /// and Silver by mod/Patches/SelfTest.xml.
    /// </summary>
    public class OnSpawnComp : ThingComp
    {
        private bool ticked;

        public override void PostSpawnSetup(bool respawningAfterLoad)
        {
            base.PostSpawnSetup(respawningAfterLoad);
            Companion.Send("ApiTestMod", "comp_spawned", new Dictionary<string, object>
            {
                ["comp"] = GetType().Name,
                ["parent"] = parent,
                ["respawning"] = respawningAfterLoad,
            });
        }

        public override void CompTick()
        {
            if (ticked)
            {
                return;
            }

            ticked = true;

            // A call the *game* makes: the replaced call runs its Rust handler here and the answer
            // travels back through the generated shim.
            Companion.Send("ApiTestMod", "replace_result", new Dictionary<string, object>
            {
                ["value"] = Bridge.AddThroughReplace(),
            });

            // A request the *game* makes: Rust is idle on a tick, so the answer comes straight back.
            Companion.Send("ApiTestMod", "companion_answer", new Dictionary<string, object>
            {
                ["answer"] = Bridge.AskRust("ping"),
            });
        }
    }

    /// <summary>
    /// The def entry that puts <see cref="OnSpawnComp"/> on a thing. RimWorld's `&lt;comps&gt;` list
    /// holds `CompProperties`, not `ThingComp`: the XML has to name this class (`compClass` points at
    /// the component), because naming the component itself makes the def fail to load.
    /// </summary>
    public class OnSpawnCompProperties : CompProperties
    {
        public OnSpawnCompProperties()
        {
            compClass = typeof(OnSpawnComp);
        }
    }

    /// <summary>
    /// An inspector tab (the "i" pane) declared in the companion and filled from Rust: the game
    /// calls FillTab, the companion asks Rust for the lines, Rust answers. Attached to Steel and
    /// Silver by mod/Patches/SelfTest.xml.
    /// </summary>
    public class RustInfoTab : ITab
    {
        public RustInfoTab()
        {
            size = new Vector2(300f, 220f);
            labelKey = "API self-test";
        }

        public override bool IsVisible => SelThing is Thing thing && thing.Spawned;

        protected override void FillTab()
        {
            Listing_Standard listing = new Listing_Standard();
            listing.Begin(new Rect(0f, 0f, size.x, size.y).ContractedBy(12f));
            foreach (string line in Contents())
            {
                listing.Label(line);
            }

            listing.End();
        }

        private IEnumerable<string> Contents()
        {
            Thing thing = SelThing;
            if (thing == null)
            {
                yield break;
            }

            string answer = Companion.Request("ApiTestMod", "itab_contents", new Dictionary<string, object>
            {
                ["thing"] = thing,
            });

            foreach (string line in answer.Split('\n'))
            {
                if (line.Length > 0)
                {
                    yield return line;
                }
            }
        }
    }
}
