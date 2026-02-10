# Desert Development Guidelines

## 1. Philosophical North Stars
- **Transparent Transpilation:** A Desert developer should be able to guess the generated Rust code. The mapping should be as direct as possible.
- **Mirage First:** If a user has to look at the generated `.rs` file to understand an error, the Mirage system has failed. Always prioritize error mapping for new features.
- **Pythonic Purity:** Maintain 4-space indentation and colon-based blocks. Avoid introducing C-style punctuation into the Desert frontend.

## 2. Technical Tips
- **Lexer Spans:** When adding new tokens, ensure they don't break the indentation stack. Synthetic `Dedent` tokens at EOF are required to satisfy the parser's block requirements.
- **Parser Precedence:** Always use the hierarchical function pattern in `nom` (e.g., `primary` -> `multiplicative` -> `additive` -> `comparison`) to maintain correct operator precedence.
- **Resolver Logic:** The `Resolver` is currently simple (capitalization-based). As the project grows, it will need to integrate with `cargo metadata` to resolve external crate types.

## 3. Testing Standards
- **Parallel Testing:** Every new AST node must have a corresponding test case in `parser.rs` (for structure) and `transpiler.rs` (for output).
- **Integration Tests:** Use `.ds` files to test the full `check` pipeline. If `rustc` rejects the transpiled code, the feature is not complete.
- **Regression:** Run `cargo test` after every change. The indentation logic is sensitive to whitespace changes in the test inputs.

## 5. Maintenance & Reflexive Updates
- **Self-Correction:** This document and `handoff.md` must be updated reflexively after any significant feature implementation or architectural change.
- **Codebase Health:** Always perform minor quality-of-life improvements (e.g., fixing warnings, improving error messages, optimizing imports) automatically when working on a task.
- **Documentation Parity:** Ensure that `overview.md` and `examples/` remain in sync with the actual language capabilities.

> [!IMPORTANT]
> Keep the project's technical documentation updated reflexively and automatically. Do not wait for explicit user requests to document new features or maintain repository health.
