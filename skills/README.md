# Agent skills in this repository

`skills/<name>/SKILL.md` is the standard layout of an [Agent Skill](https://agentskills.io), and both ecosystems
that install skills already look here: the [`skills`](https://www.npmjs.com/package/skills) CLI discovers a
repository's `skills/` directory, and pnpm (≥ 12.11) links `skills/<name>/SKILL.md` from an approved package one
level deep. Keeping the skill in the repository therefore means an agent that already reads this checkout can use
it, while anything else can install it from here.

Currently shipped:

| Skill | What it covers |
|---|---|
| [`rimworld-rs-tooling`](rimworld-rs-tooling/SKILL.md) | The `cargo xtask` surface, the gates (`cargo xtask test`, `tools/abi-smoke`), where each document lives, the feature checklist, and the traps. |

## Route A — the `skills` CLI, through pnpm

The CLI is the package [`skills`](https://www.npmjs.com/package/skills) (Vercel Labs; it supports Command Code,
Claude Code, Codex, Cursor and ~75 more agents). pnpm's equivalent of `npx` is `dlx`, so no global install is
needed:

```bash
# 1. see what this repository offers (nothing is written)
pnpm dlx skills add ./skills --list

# 2. install the skill for Command Code, project scope (default)
pnpm dlx skills add ./skills --skill rimworld-rs-tooling -a command-code -y

#    ... or globally, for every project on this machine
pnpm dlx skills add ./skills --skill rimworld-rs-tooling -g -a command-code -y

#    ... or for several agents at once
pnpm dlx skills add ./skills --skill rimworld-rs-tooling -a command-code -a claude-code -y
```

- **Where it lands**: project scope → `./<agent>/skills/` (Command Code: `.commandcode/skills/`), global scope →
  `~/<agent>/skills/` (Command Code: `~/.commandcode/skills/`).
- **Symlinks vs copies**: the CLI symlinks to a canonical copy by default; on Windows without symlink support add
  `--copy`, which writes independent files instead.
- **Sources**: a local path (`./skills`), a GitHub shorthand (`owner/repo`), a full URL, or a direct path to one
  skill in a repository
  (`pnpm dlx skills add https://github.com/KrzysztofZawisla/rimworld-rs/tree/main/skills/rimworld-rs-tooling`).
- **Check, update, remove**: `pnpm dlx skills list`, `pnpm dlx skills update rimworld-rs-tooling`,
  `pnpm dlx skills remove rimworld-rs-tooling`.
- `--all`, `--agent '*'` and `--skill '*'` exist for non-interactive installs of everything; `INSTALL_INTERNAL_SKILLS=1`
  reveals skills marked `metadata.internal: true` (this one is not).

## Route B — pnpm's own agent skills (≥ 12.11)

pnpm links the skills of an approved **npm package dependency** into the agent directories of the project, without
the CLI. It requires the skill to travel inside a package that has a `package.json`:

```bash
# once, if you want this repository to be installable as a package: add a minimal manifest
cat > package.json <<'JSON'
{ "name": "rimworld-rs-skills", "version": "0.1.0", "private": true }
JSON
```

```bash
# then, in the project that wants the skill (a git or tarball dependency is what pnpm keys the approval to)
pnpm add -D git+https://github.com/KrzysztofZawisla/rimworld-rs.git
pnpm approve          # reviews the pending package and records the decision
```

`pnpm approve` writes the approval into `pnpm-workspace.yaml`:

```yaml
permissions:
  rimworld-rs-skills:
    skills: true
```

- **Where it lands**: the agent skill directories that already exist in the workspace root (`.claude/skills/`,
  `.cursor/skills/`, `.agents/skills/`, …); with an agent environment variable set (e.g. `AI_AGENT=1`) its
  directory is created. To choose directories yourself — for example
  Command Code's `.commandcode/skills` — set them explicitly:

  ```yaml
  # pnpm-workspace.yaml
  skills:
    dirs:
      - .commandcode/skills
  ```

- Links are named `pnpm-<package>-<skill>` and pnpm adds a `pnpm-*` rule to a `.gitignore` in each directory,
  because the links point into `node_modules`.
- **Limits**: `pnpm dlx`, global installs and `pnpm deploy` do **not** link skills; approval covers every skill
  of that package for later versions; a skill approved with no directory to link into fails the install.
- This repository has no `package.json` today (it is a Cargo workspace), which is why Route A works as-is and
  Route B starts with the one-line manifest above.

## Which route to pick

| Situation | Route |
|---|---|
| An agent working in this checkout | Nothing to install: point it at `skills/rimworld-rs-tooling/SKILL.md` |
| One agent, on this machine | A, with `-g -a <agent> -y` |
| A team sharing a project | A, project scope (the CLI writes into `./<agent>/skills/`), or B if the project is already a pnpm workspace |
| Skills shipped with a package you publish | B |
