# Desert Project Handoff

## Repository Status

Desert is a working prototype compiler frontend written in Rust. It lexes/parses `.ds` source, transpiles to Rust, and can run Rust type checks while translating diagnostics into Desert-friendly wording.

Project direction now favors semantic simplification over compatibility. Breaking grammar/lowering changes are expected when they reduce duplicate forms.

Core quality checks currently pass:

- `cargo test`
- `cargo clippy --all-targets --all-features`

## Architecture

- `src/lexer.rs`: Logos-based lexer with indentation stack and virtual `Indent`/`Dedent` tokens.
- `src/ast.rs`: AST definitions for statements, expressions, and types with source spans.
- `src/parser.rs`: Recursive-descent parser over token spans.
- `src/transpiler.rs`: AST to Rust code generation plus source map creation.
- `src/resolver.rs`: Lightweight scoped type/value symbol tracker for unified-dot lowering.
- `src/sourcemap.rs`: Rust-to-Desert mapping with source file + line locations.
- `src/mirage.rs`: Rust diagnostic translation to Desert terminology.
- `src/main.rs`: CLI entry point (`transpile`, `check`) and diagnostics plumbing.
- `src/main.rs`: CLI entry point (`transpile`, `check`, `graph`) and diagnostics plumbing.

## Implemented Language Surface

- Declarations: `let`, `mut`
- Control flow: `if/else`, `for`, `match`
- Definitions: `def`, `struct`, `protocol`, `impl`
- Expressions: literals, calls, member access, generic calls, indexing, assignment
- Imports: top-level `import` statements for project files
- Ownership/error syntax: `move`, `&`, `~`, `?`, `!!`
- Macros: `$name(...)` with `$print` -> `println!`
- `pyimport` blocks: parsed and emitted as Rust comments

## Implemented Project Surface

- CLI now accepts either a single `.ds` file or a project directory for `transpile`/`check`.
- Project directories require `desert.toml` or `Desert.toml`.
- Entrypoint resolution uses `[package].entry` when provided, defaulting to `src/main.ds`.
- Project mode resolves top-level imports recursively (relative to importing file), defaults missing import extensions to `.ds`, and rejects import cycles.
- `desert graph <project_dir>` prints the resolved import/topological load order used for compilation.

## Recent Cleanup

- Improved CLI diagnostics with line/column parser/lexer errors.
- `check` now uses unique temp directories, `rustc --emit=metadata`, and isolated rustc outputs in temp dirs.
- Cleaned parser generic-call flow and pyimport token rendering.
- Simplified transpiler internals and removed placeholder block output path.
- Added `Default` implementations and resolved Clippy warnings.
- Added interpolation-safe `$print` lowering and struct-constructor lowering.
- Added protocol parameter lowering to Rust `impl Trait`.
- Added `@` lowering to generated `desert_matmul(...)` helpers.
- Added `desert check` integration coverage for all examples in `tests/check_examples.rs`.
- Added negative `desert check` integration fixtures to assert translated rustc diagnostics for type mismatch, mutability borrow errors, and method-resolution failures, plus parser/lexer location errors.
- Added explicit Mirage hints keyed by rustc error codes (`E0308`, `E0596`, `E0599`) with unit tests.
- Replaced resolver capitalization heuristics with scoped symbol tracking for unified-dot lowering, including shadowing-aware behavior.
- Removed uppercase fallback in receiver classification; static `Type.method` lowering now requires declared or built-in type symbols (for example, `Box`).
- Added pre-Rust semantic validation for mutability-sensitive forms so `move x` and `~x` fail fast with Desert line/column errors when `x` is not declared `mut`, and now also reject non-place operands such as `move foo()` or `~foo()`.
- Extended mutability-sensitive prechecks so `move`/`~` on member/index places (`obj.field`, `items[i]`) accept unique-reference write-through roots, matching assignment write-through behavior.
- Added pre-Rust assignment validation so `lhs = rhs` now fails early unless `lhs` is a place expression, the root binding exists, and write access is valid (`mut` root or unique-reference write-through for member/index assignment). Struct constructor named arguments (`Type(field=value)`) are handled explicitly as constructor syntax, not assignment.
- Added pre-Rust struct-constructor argument validation so `Type(...)` now fails fast on unknown/duplicate named fields, positional overflow, and missing required fields instead of deferring to rustc.
- Removed statement-level borrow declarations (`ref`, `mut ref`) from AST/parser/transpiler. Borrow binding is now expression-only (`let a = &x`, `let b = ~x`), and `ref` is no longer a reserved keyword.
- Added `import` parsing and project graph loading so multi-file projects compile from a single entrypoint.
- Added file-aware diagnostic mapping so `desert check <project_dir>` now reports imported-module paths with Desert line numbers (for example, `src/util/math.ds:2`).

## Known Gaps

- No project-level dependency management yet.
- No package/dependency management yet beyond local file imports.
- Rust diagnostics still map by line (not precise columns) after transpilation.
- Resolver is heuristic, not semantic.
- Mirage translations are simple string rewrites.
- Matmul lowering currently targets specific float vector/matrix shapes.

## Recommended Next Steps

1. Improve file-aware source mapping from file+line to source-accurate file+line+column diagnostics.
2. Add basic tooling primitives first (formatter scaffold, cache-key groundwork for faster checks, CI command split).
3. Expand Mirage hints with ownership/lifetime-oriented guidance as diagnostics layer work.
4. Convert `pyimport` into executable interop scaffolding later.
