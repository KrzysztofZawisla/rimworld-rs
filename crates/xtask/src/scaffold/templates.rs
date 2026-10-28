pub(crate) const LIB_TEMPLATE: &str = r#"//! {display} - a RimWorld mod written in Rust.

use rimworld_rs::prelude::*;

#[derive(Default)]
struct Plugin;

impl Mod for Plugin {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.log(format!("{display}: on_init"));
    }

    fn on_defs_loaded(&mut self, ctx: &mut Context) {
        match ctx.def_names("ThingDef") {
            Ok(defs) => ctx.log(format!("{display}: {count} thing defs loaded", count = defs.len())),
            Err(error) => ctx.error(format!("{display}: cannot read defs: {error}")),
        }
    }

    fn on_game_started(&mut self, ctx: &mut Context) {
        if let Err(error) = ctx.send_letter(LetterKind::Neutral, "{display}", "The mod is alive.") {
            ctx.error(format!("{display}: cannot send a letter: {error}"));
        }
    }

    fn on_tick(&mut self, _ctx: &mut Context) {
        // Runs every game tick; keep it cheap or use ctx.set_tick_interval(250).
    }
}

rimworld_rs::export_mod!(Plugin);
"#;

/// The companion project of a fresh mod: managed game types a native library cannot define.
pub(crate) const COMPANION_PROJECT_TEMPLATE: &str = r#"<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net9.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <ImplicitUsings>disable</ImplicitUsings>
    <Nullable>disable</Nullable>
    <AssemblyName Condition="'$(AssemblyName)' == ''">{name}Companion</AssemblyName>
    <RootNamespace>{name}Companion</RootNamespace>
    <AppendTargetFrameworkToOutputPath>false</AppendTargetFrameworkToOutputPath>
    <CopyLocalLockFileAssemblies>false</CopyLocalLockFileAssemblies>
    <DebugType>none</DebugType>
    <GenerateDocumentationFile>false</GenerateDocumentationFile>
    <SatelliteResourceLanguages>en</SatelliteResourceLanguages>
    <NoWarn>$(NoWarn);CS1701;CS1702;MSB3277;MSB3246;MSB3247</NoWarn>

    <!-- The game runs on .NET 10 (a WaylithCLR fork) while a local SDK may be 9.x, so the companion
         is compiled directly against the game's own assemblies; `cargo xtask build` passes the game
         folder as -p:RimWorldDir. Game assemblies are never part of this repository. -->
    <!-- Both of these are what keeps a *build* from mixing the SDK's framework assemblies into the
         game's own (that produces CS1705). `dotnet format` needs a framework reference to build a
         compilation for its analyzer stages: without one it prints "Required references were not
         loaded for project ...", exits 0, and checks nothing but whitespace. RimWorldAnalysisPass=1
         re-enables them for that pass alone; `cargo xtask lint` sets it. -->
    <DisableImplicitFrameworkReferences Condition="'$(RimWorldAnalysisPass)' != '1'">true</DisableImplicitFrameworkReferences>
    <NoStdLib Condition="'$(RimWorldAnalysisPass)' != '1'">true</NoStdLib>
    <GenerateTargetFrameworkAttribute>false</GenerateTargetFrameworkAttribute>

    <RimWorldDir Condition="'$(RimWorldDir)' == '' AND Exists('$(MSBuildThisFileDirectory)..\..\..\game-refs\Assembly-CSharp.dll')">$(MSBuildThisFileDirectory)..\..\..\game-refs</RimWorldDir>
    <RimWorldDir Condition="'$(RimWorldDir)' == '' AND Exists('$(MSBuildThisFileDirectory)..\..\game-refs\Assembly-CSharp.dll')">$(MSBuildThisFileDirectory)..\..\game-refs</RimWorldDir>
  </PropertyGroup>

  <Target Name="ValidateRimWorldReferences" BeforeTargets="BeforeCompile">
    <Error Condition="'$(RimWorldDir)' == ''"
           Text="RimWorld reference assemblies were not found. Run `cargo xtask setup` to copy them into game-refs/, or build with -p:RimWorldDir=&lt;game folder&gt;." />
    <Error Condition="'$(RimWorldDir)' != '' AND !Exists('$(RimWorldDir)\Assembly-CSharp.dll')"
           Text="'$(RimWorldDir)' does not contain Assembly-CSharp.dll; point RimWorldDir at the RimWorld installation folder (or re-run `cargo xtask setup`)." />
  </Target>

  <ItemDefinitionGroup>
    <!-- The game PROVIDES these assemblies at runtime; a mod must never ship its own copies. -->
    <Reference>
      <Private>false</Private>
      <SpecificVersion>false</SpecificVersion>
    </Reference>
  </ItemDefinitionGroup>

  <ItemGroup Condition="'$(RimWorldDir)' != ''">
    <Reference Include="$(RimWorldDir)\Assembly-CSharp.dll" />
    <Reference Include="$(RimWorldDir)\UnityStubs.dll" />
    <Reference Include="$(RimWorldDir)\RimWorld.dll" />
    <Reference Include="$(RimWorldDir)\System.Private.CoreLib.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.Extensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.InteropServices.dll" />
    <Reference Include="$(RimWorldDir)\System.Runtime.CompilerServices.Unsafe.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.Concurrent.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.NonGeneric.dll" />
    <Reference Include="$(RimWorldDir)\System.Collections.Specialized.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.Expressions.dll" />
    <Reference Include="$(RimWorldDir)\System.Linq.Queryable.dll" />
    <Reference Include="$(RimWorldDir)\System.Memory.dll" />
    <Reference Include="$(RimWorldDir)\System.ObjectModel.dll" />
    <Reference Include="$(RimWorldDir)\System.Globalization.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.Json.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.Encodings.Web.dll" />
    <Reference Include="$(RimWorldDir)\System.Text.RegularExpressions.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Thread.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Tasks.dll" />
    <Reference Include="$(RimWorldDir)\System.Threading.Tasks.Parallel.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Extensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Primitives.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.TypeExtensions.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.ILGeneration.dll" />
    <Reference Include="$(RimWorldDir)\System.Reflection.Emit.Lightweight.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.FileSystem.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.Compression.dll" />
    <Reference Include="$(RimWorldDir)\System.IO.Compression.ZipFile.dll" />
    <Reference Include="$(RimWorldDir)\System.Diagnostics.Debug.dll" />
    <Reference Include="$(RimWorldDir)\System.Diagnostics.Process.dll" />
    <Reference Include="$(RimWorldDir)\System.Xml.ReaderWriter.dll" />
    <Reference Include="$(RimWorldDir)\System.Xml.XDocument.dll" />
    <Reference Include="$(RimWorldDir)\System.Security.Cryptography.dll" />
    <Reference Include="$(RimWorldDir)\Microsoft.CSharp.dll" />
    <Reference Include="$(RimWorldDir)\netstandard.dll" />
    <!-- Need another framework assembly? Add a line like the ones above:
         <Reference Include="$(RimWorldDir)\System.Foo.dll" /> -->
  </ItemGroup>

  <ItemGroup Condition="'$(BridgeAssembly)' != ''">
    <!-- The bridge of this mod; Rust calls into it and it calls back into Rust (Companion.Send). -->
    <Reference Include="$(BridgeAssembly)">
      <HintPath>$(BridgePath)</HintPath>
      <Private>false</Private>
    </Reference>
  </ItemGroup>

</Project>
"#;

/// The sample companion source: one static helper wired to Rust.
pub(crate) const COMPANION_TEMPLATE: &str = r#"using System;
using System.Collections.Generic;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// A companion assembly: the managed half of a Rust mod. Rust cannot create managed game types
    /// (ThingComp, HediffComp, JobDriver, ITab, Window, ...), so those live here; the Rust side
    /// talks to this assembly with `ctx.companion_call(...)` and receives events with
    /// `fn on_companion_event(...)`. See docs/companion.md.
    /// </summary>
    public static class Bridge
    {
        /// <summary>Called from Rust: ctx.companion_call("{namespace}.Bridge", "Describe", &[json!("steel")]).</summary>
        public static string Describe(string what)
        {
            return "the companion was asked about '" + what + "' at " + DateTime.UtcNow.ToString("HH:mm:ss");
        }

        /// <summary>Asks the Rust side something and uses its answer (empty when Rust returns none).</summary>
        public static string AskRust(string name, object payload)
        {
            return Companion.Request("{name}", name, payload);
        }
    }
}
"#;

/// A ThingComp sample (`cargo xtask new --comp`): how a managed type reports to Rust.
pub(crate) const THING_COMP_TEMPLATE: &str = r#"using System.Collections.Generic;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// Reports its thing to Rust when it spawns and when it ticks once. Attach it to a def with the
    /// patch in mod/Patches/{name}Comp.xml, then answer in Rust:
    ///
    /// <code>
    /// fn on_companion_event(&amp;mut self, ctx: &amp;mut Context, _source: &amp;str, name: &amp;str, payload: &amp;Value) -&gt; Option&lt;String&gt; {
    ///     if name == "comp_spawned" {
    ///         let parent = payload.get("parent").and_then(ObjRef::from_value);
    ///         ctx.log(format!("a thing spawned: {parent:?}"));
    ///     }
    ///     None
    /// }
    /// </code>
    /// </summary>
    public class OnSpawnComp : ThingComp
    {
        private bool ticked;

        public override void PostSpawnSetup(bool respawningAfterLoad)
        {
            base.PostSpawnSetup(respawningAfterLoad);
            Companion.Send("{name}", "comp_spawned", new Dictionary<string, object>
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
            Companion.Send("{name}", "comp_ticked", new Dictionary<string, object>
            {
                ["parent"] = parent,
            });
        }
    }

    /// <summary>
    /// The entry that puts <see cref="OnSpawnComp"/> on a def: a `&lt;comps&gt;` list holds
    /// `CompProperties`, not `ThingComp`, so the patch in mod/Patches names *this* class.
    /// </summary>
    public class OnSpawnCompProperties : CompProperties
    {
        public OnSpawnCompProperties()
        {
            compClass = typeof(OnSpawnComp);
        }
    }
}
"#;

/// A JobDriver sample (`cargo xtask new --job`): a job a pawn can be given.
pub(crate) const JOB_DRIVER_TEMPLATE: &str = r#"using System.Collections.Generic;
using Verse;
using Verse.AI;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// A job that walks a pawn to the job's target cell. Start it from Rust with a job the pawn
    /// should do, for example:
    ///
    /// <code>
    /// // build a job of this driver for a pawn and hand it over
    /// let job = ctx.job_for(pawn.handle(), "MyModCompanion.WalkToCellJobDriver", &amp;[json!({"cell": [x, z]})])?;
    /// ctx.pawn_take_job(pawn.handle(), &amp;job)?;
    /// </code>
    ///
    /// (the comp and job helpers of the SDK are described in docs/companion.md)
    /// </summary>
    public class WalkToCellJobDriver : JobDriver
    {
        public override bool TryMakePreToilReservations(bool errorOnFailed)
        {
            return true;
        }

        protected override IEnumerable<Toil> MakeNewToils()
        {
            // Walk to the target cell, then end the job.
            yield return Toils_Goto.GotoCell(TargetIndex.A, PathEndMode.OnCell);
        }
    }
}
"#;

/// An ITab sample (`cargo xtask new --itab`): a tab in the inspect pane, filled from Rust.
pub(crate) const INSPECTOR_TAB_TEMPLATE: &str = r#"using System.Collections.Generic;
using RimWorld;
using UnityEngine;
using Verse;
using RustBridge;

namespace {namespace}
{
    /// <summary>
    /// Adds a tab to the inspect pane for every thing whose def names it:
    ///
    /// <code>
    /// &lt;Operation Class="PatchOperationAdd"&gt;
    ///   &lt;xpath&gt;/Defs/ThingDef[defName="Steel"]&lt;/xpath&gt;
    ///   &lt;value&gt;&lt;inspectorTabs&gt;&lt;li&gt;{namespace}.RustInfoTab&lt;/li&gt;&lt;/inspectorTabs&gt;&lt;/value&gt;
    /// &lt;/Operation&gt;
    /// </code>
    ///
    /// The content comes from Rust: answer the `itab_contents` event with the lines to show,
    /// separated by newlines (see `Mod::on_companion_event`).
    /// </summary>
    public class RustInfoTab : ITab
    {
        public RustInfoTab()
        {
            size = new Vector2(320f, 240f);
            labelKey = "{name} info";
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

            string answer = Companion.Request("{name}", "itab_contents", new Dictionary<string, object>
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
"#;

/// The XML patch that attaches the sample comp (`cargo xtask new --comp`).
///
/// The root element must be `<Patch>`: RimWorld logs "Unexpected document element in patch XML"
/// and skips a whole file whose root is anything else, so a wrong root silently removes the
/// feature instead of failing. `cargo xtask test` checks every patch file in the repository.
pub(crate) const COMP_PATCH_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8" ?>
<Patch>
	<!-- Attaches the companion's ThingComp to Steel, so the mod sees when a thing spawns. Change
	     the def (and add more operations) as needed. Two rules the game only reports in its log:
	     - the value goes on the *def* and creates <comps>: a def without a <comps> node cannot be
	       extended through /comps;
	     - a <comps> entry names its *CompProperties* class (the one holding compClass), never the
	       ThingComp itself - a component in that list stops the def from loading and the game then
	       resets the mod list on the next start. -->
	<Operation Class="PatchOperationAdd">
		<xpath>/Defs/ThingDef[defName="Steel"]</xpath>
		<value>
			<comps>
				<li Class="{namespace}.OnSpawnCompProperties" />
			</comps>
		</value>
	</Operation>
</Patch>
"#;
