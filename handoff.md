# Desert Project Handoff

## Repository Status

Desert is a working prototype compiler frontend written in Rust. It lexes/parses `.ds` source, transpiles to Rust, and can run Rust type checks while translating diagnostics into Desert-friendly wording.

Core quality checks currently pass:

- `cargo test`
- `cargo clippy --all-targets --all-features`

## Architecture

- `src/lexer.rs`: Logos-based lexer with indentation stack and virtual `Indent`/`Dedent` tokens.
- `src/ast.rs`: AST definitions for statements, expressions, and types with source spans.
- `src/parser.rs`: Recursive-descent parser over token spans.
- `src/transpiler.rs`: AST to Rust code generation plus source map creation.
- `src/resolver.rs`: Lightweight type/name heuristic for unified-dot lowering.
- `src/sourcemap.rs`: Line-based Rust-to-Desert mapping.
- `src/mirage.rs`: Rust diagnostic translation to Desert terminology.
- `src/main.rs`: CLI entry point (`transpile`, `check`) and diagnostics plumbing.

## Implemented Language Surface

- Declarations: `let`, `mut`, `ref`, `mut ref`
- Control flow: `if/else`, `for`, `match`
- Definitions: `def`, `struct`, `protocol`, `impl`
- Expressions: literals, calls, member access, generic calls, indexing, assignment
- Ownership/error syntax: `move`, `&`, `~`, `?`, `!!`
- Macros: `$name(...)` with `$print` -> `println!`
- `pyimport` blocks: parsed and emitted as Rust comments

## Recent Cleanup

- Improved CLI diagnostics with line/column parser/lexer errors.
- `check` now uses unique temp directories, respects Rust 2024 edition, and returns proper failures.
- Cleaned parser generic-call flow and pyimport token rendering.
- Simplified transpiler internals and removed placeholder block output path.
- Added `Default` implementations and resolved Clippy warnings.

## Known Gaps

- No project-level dependency management yet.
- Resolver is heuristic, not semantic.
- Mirage translations are simple string rewrites.
- `@` is tokenized but currently lowered to `*`.
- Source map is line-based only.

## Recommended Next Steps

1. Add end-to-end tests for `desert check` against files in `examples/`.
2. Introduce a richer resolver pass with scoped symbols.
3. Expand Mirage translation with targeted borrow-checker hints.
4. Decide and implement a concrete lowering strategy for `pyimport` and `@`.
