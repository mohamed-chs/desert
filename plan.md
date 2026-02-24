# Desert Roadmap

This roadmap assumes deliberate breaking changes when they reduce syntax overlap or compiler branching.

## Phase 1: Semantic Core Simplification

- Keep one canonical binding path (`let`/`mut` only). (Done for borrow declarations: removed statement-level `ref`/`mut ref`)
- Keep ownership operations expression-based (`move`, `&`, `~`) with explicit pre-emit validation.
- Continue pruning overlapping forms that lower to identical Rust.

## Phase 2: Resolution and Type Semantics

- Extend resolver from receiver classification into symbol/type validation.
- Add explicit semantic checks for callable/member/index targets before Rust emit. (Partially done: assignment target semantics now checked pre-Rust, including constructor named-arg disambiguation.)
- Keep precedence and lowering conventions explicit and test-backed.

## Phase 3: Diagnostics and Mirage

- Expand rustc code-specific hints beyond `E0308`/`E0596`/`E0599`.
- Improve source remapping precision from line-based toward span-aware mapping.
- Emit direct “Desert-level” fix suggestions for common ownership/type failures.

## Phase 4: Interop and Runtime Surface

- Replace `pyimport` comment passthrough with executable interop scaffolding.
- Define a minimal `desert_core` surface aligned with current lowering rules.
- Generalize `@` lowering beyond current float vector/matrix helper coverage.

## Phase 5: Tooling

- Add formatting and style normalization.
- Add language-server-grade parse/diagnostic services once semantics settle.

## Delivery Notes

- Behavior changes ship first; tests/docs track the shipped semantics in the same pass.
- Backward compatibility is not a goal unless explicitly reintroduced.
