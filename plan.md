# Desert Roadmap

This roadmap follows current project direction: prioritize project workflow and basic tooling first, defer runtime/domain specialization until the core developer loop is stable.

## Active Priorities

1. Multi-file project workflow: Yes
2. Full semantic/type system rewrite now: No
3. Diagnostics productization: Maybe, staged
4. Runtime/interop overhaul: Later
5. Tooling: Yes, basics first
6. Domain specialization: Later

## Phase A: Project Workflow Foundation (Current)

- Accept project directories in CLI (`desert.toml`/`Desert.toml`) and resolve an explicit entrypoint. (Done)
- Implement first-pass module/import graph loader with cycle detection. (Done)
- Add direct execution workflow (`desert run <file|project> [-- args...]`). (Done)
- Add zero-config project scaffolding (`desert new <path>`). (Done)
- Add formatter scaffold and stable style output. (Done)
- Add preflight environment/project validation (`desert doctor`). (Done)
- Keep transpilation predictable and readable while introducing cross-file compilation order.
- Preserve current lowering conventions while scaling from single-file to project mode.

## Phase B: Tooling Basics (Parallel to Phase A)

- Add project-graph check mode and cache-key groundwork for faster `check` loops. (Graph command done)
- Add CI-facing staged `check` support that separates syntax, semantic prechecks, and rustc-backed checking. (Done)

## Phase C: Diagnostics Upgrade (Staged, Optional While A/B Run)

- Move from line-level mapping toward span-aware locations. (File+line+statement-start-column mapping is done; token-accurate columns remain.)
- Expand Mirage coverage for common ownership/type rustc families.
- Add explicit fix-style hints for recurring Desert authoring mistakes.

## Deferred Phases

- Runtime/interop system (`pyimport` replacement, `desert_core`) after project/tooling basics are stable.
- Domain-focused language shaping after real project usage data exists.

## Delivery Notes

- Ship behavior changes first; tests/docs follow in the same pass.
- Breaking changes remain acceptable when they reduce overlap or compiler branching.
- Keep examples executable and checkable with current compiler behavior.
- Recent shipped behavior: file-mode import graph resolution now loads top-level `import` dependencies (with cycle detection) for `check`/`transpile`/`run`, not just project-directory inputs.
- Recent shipped behavior: semantic validation now checks direct `def` call arity (including forward local defs and generic-call syntax) before rustc.
- Recent shipped behavior: `transpile`/`check`/`run`/`fmt`/`graph` now default to current directory when input is omitted.
