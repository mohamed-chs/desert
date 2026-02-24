# Desert Roadmap

This roadmap focuses on shipping a stable core before expanding syntax.

## Phase 1: Compiler Stability

- Tighten parser/transpiler error messages (line/column + useful context).
- Add golden tests for transpilation output.
- Add integration tests for `desert check` on example programs. (Done for current `examples/*.ds`)
- Add integration tests for `desert check` failure paths and translated diagnostics. (Type mismatch + parser + lexer + borrow mutability + method-resolution fixtures added)
- Keep `cargo test` and `cargo clippy` clean.

## Phase 2: Semantic Resolution

- Replace capitalization heuristics with scoped symbol tracking.
- Improve generic resolution for static functions vs methods.
- Validate `move` usage and borrow forms more explicitly before Rust emit. (Now rejects `move x` / `~x` when `x` is not declared `mut`, before rustc)

## Phase 3: Mirage Quality

- Improve diagnostic rewrites beyond token replacement.
- Add explicit, code-driven hints for common rustc diagnostics. (Initial `E0308`/`E0596`/`E0599` support added)
- Map more Rust concepts back to Desert phrasing.
- Include actionable hints in translated errors.

## Phase 4: Interop and Standard Surface

- Define a minimal `desert_core` module set.
- Move `pyimport` from comment passthrough to real interop scaffolding.
- Generalize matrix/tensor operator lowering (`@`) beyond current float vector/matrix helper path.

## Phase 5: Tooling

- Add formatter pass for canonical Desert style.
- Evaluate an LSP path once grammar and diagnostics stabilize.

## Delivery Notes

- Keep feature work test-first where possible.
- Avoid broad syntax expansion until diagnostics and resolution are reliable.
- Update docs/examples with each behavior change so drift stays low.
- In normal forward-progress turns, ship behavior first; avoid test/docs-only iterations unless explicitly requested.
