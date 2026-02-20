# Desert Development Guidelines

## Principles

- Prefer predictable transpilation over clever syntax tricks.
- Keep generated Rust readable; it is still a debugging escape hatch.
- Add language features only with parser + transpiler tests together.

## Code Guidelines

- Preserve indentation-sensitive lexer behavior; whitespace regressions are easy.
- Keep parser precedence explicit (`primary -> multiplicative -> additive -> comparison -> assignment`).
- Treat `Resolver` as a temporary strategy layer; avoid burying semantic rules in transpiler string code.
- Keep Mirage message rewrites explicit and conservative.

## Testing Expectations

- Run `cargo test` for every functional change.
- Run `cargo clippy --all-targets --all-features` for quality checks.
- Add regression tests for every parser or transpiler bug fix.
- Prefer small `.ds` fixtures that model real usage patterns.

## Documentation Hygiene

- Keep `overview.md` focused on what exists now, not aspirational features.
- Keep `handoff.md` operational: architecture, status, risks, next tasks.
- When behavior changes, update examples and docs in the same patch.
