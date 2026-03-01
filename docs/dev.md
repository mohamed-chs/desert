# Desert Developer Guide

## Setup

```bash
cargo build
cargo test
```

## Repo Map

- `src/lexer.rs`: tokenization + indentation (`Indent`/`Dedent`)
- `src/parser.rs`: recursive-descent parser
- `src/ast.rs`: AST nodes + spans
- `src/resolver.rs`: scoped symbol tracking for dot resolution
- `src/transpiler.rs`: Desert -> Rust lowering
- `src/sourcemap.rs`: Rust-to-Desert location mapping
- `src/mirage.rs`: rustc diagnostic rewrite
- `src/main.rs`: CLI wiring and pipeline
- `examples/*.ds`: executable fixtures
- `tests/`: regression and integration coverage

## Change Workflow

1. Make one behavior change.
2. Add/update regression tests for parser/transpiler bugs.
3. Run validation gates.
4. Update docs/examples only as needed for shipped behavior.

Keep commits atomic and behavior-first.

## Validation Gates

For compiler behavior changes:

```bash
cargo test
for f in examples/*.ds; do cargo run -- check "$f"; done
```

For docs-only edits, full validation is optional.

## Guardrails

- Preserve indentation-sensitive lexer behavior.
- Keep precedence explicit:
  `primary -> multiplicative -> additive -> comparison -> assignment`
- Preserve lowering conventions unless intentionally changing them:
  - `move expr` -> `std::mem::take(&mut expr)`
  - receiver lowering (`self` -> `&self`, `mut self` -> `&mut self`)
  - constructor call rewrite (`Type(...)` -> struct literal when applicable)
  - `@` lowers through generated matmul helpers
- Keep semantic rules in semantic/resolver layers, not ad-hoc string rewrites.

## Fast Debug Paths

- Parser issue: run `desert check --stage syntax`.
- Semantic issue: run `desert check --stage semantic`.
- Rust lowering/typing issue: run full `desert check` and inspect Mirage output.

## Documentation Targets

- `overview.md`: current language behavior only.
- `handoff.md`: operational state, risks, and next tasks.
- `plan.md`: done vs pending, no stale roadmap items.
