# AGENTS.md

## Core Direction

- Optimize for forward progress and language quality over backward compatibility.
- Backward compatibility is optional by default; breaking changes are acceptable when they simplify semantics or reduce compiler complexity.
- Prefer predictable transpilation over clever syntax tricks.
- Keep generated Rust readable; it is still the debugging escape hatch.

## Iteration Workflow

- Work in small green steps: change -> test -> commit.
- Keep commits atomic: one logical concern per commit (behavior, tests, docs, examples split when practical).
- Make frequent commits during active implementation, not only at the end.

## Validation Gates

- For functional changes, run `cargo test`.
- Before declaring completion, run `desert check` over all `examples/*.ds`.
- Add regression tests for every parser/transpiler bug fix.
- Prefer small `.ds` fixtures that model real usage patterns.

## Language and Transpiler Guardrails

- Preserve indentation-sensitive lexer behavior; whitespace regressions are easy.
- Keep parser precedence explicit (`primary -> multiplicative -> additive -> comparison -> assignment`).
- Preserve and test current lowering conventions unless intentionally changing them:
- `move expr` lowering (`std::mem::take(&mut expr)`).
- `self` receiver lowering (`self` -> `&self`, `mut self` -> `&mut self` where applicable).
- Struct constructor rewriting from call-style to struct literals.
- `@` lowering via generated matmul helpers, not silent remapping to plain `*`.
- Treat `Resolver` as a temporary strategy layer; avoid burying semantic rules in transpiler string code.
- Keep Mirage message rewrites explicit and conservative.

## Examples Policy

- Treat `examples/*.ds` as executable fixtures, not just documentation.
- Keep examples checkable with current compiler behavior.
- If examples are edited, update integration coverage (`tests/check_examples.rs`) in the same pass.

## Docs and Repo Hygiene

- Keep `overview.md` focused on current behavior, not aspirational features.
- Keep `handoff.md` operational: architecture, status, risks, next tasks.
- Keep `plan.md` aligned with what is done vs pending.
- When behavior changes, update examples and docs in the same pass.
- Do not leave temp or compiler artifacts in repo root before finishing.

## External Reference Rule

- When changing compiler/language conventions, verify with primary documentation first (Rust reference/rustc docs), then implement.
