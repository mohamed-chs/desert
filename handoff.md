# Desert Project Handoff

## 1. System Overview
**Desert** is an indentation-based systems programming language that transpiles to Rust. It aims to combine Python's ergonomics with Rust's performance and memory safety. The toolchain consists of a custom lexer/parser and the **Mirage Engine** for error translation.

## 2. Core Architecture
- **Lexer (`src/lexer.rs`):** Uses `logos` for tokenization. It features a custom wrapper that handles indentation by emitting virtual `Indent` and `Dedent` tokens and filtering out blank-line noise.
- **AST (`src/ast.rs`):** A clean representation of Desert's semantics. Statements are wrapped in a struct that preserves `Range<usize>` spans for source mapping.
- **Parser (`src/parser.rs`):** A recursive descent parser built with `nom`. It handles precedence for arithmetic and comparison operators, as well as complex constructs like nested `MemberAccess` and generic calls.
- **Transpiler (`src/transpiler.rs`):** Converts the Desert AST into idiomatic Rust. It performs basic type mapping (e.g., `List` -> `Vec`) and utilizes a `Resolver` to handle the "Unified Dot."
- **Source Mapping (`src/sourcemap.rs`):** Maintains a line-by-line mapping from the generated `.rs` file back to the original `.ds` file.
- **Mirage Engine (`src/mirage.rs`):** Intercepts JSON diagnostics from the Rust compiler, rewrites them using Desert terminology (e.g., replacing `&mut` with `~`), and remaps line numbers.

## 3. Current Progress
- **Syntax:** Support for `let`/`mut` bindings, `def` with type annotations, `if`/`else` statements, `for` loops, `struct` definitions, `protocol` (trait) definitions, `impl` blocks (inherent and trait), `pyimport` blocks, `match` statements, indexing (`expr[index]`), and memory syntax (`move`, `ref`, `mut ref`).
- **Unified Dot:** Successfully distinguishes between static calls (`Path.new`) and instance methods (`path.exists`) based on naming conventions (capitalization).
- **Arithmetic/Comparison:** Full support for standard operators including the AI-specific `@` (matrix multiplication) and `Indexing`.
- **Error Handling:** Full implementation of `?` (propagation) and `!!` (unwrap) operator support in the parser and transpiler.
- **Python Interop:** Enhanced `pyimport` block to capture tokens and generate structured comments in the Rust output.
- **CLI:** A functional `desert` binary with `transpile` and `check` commands.
- **Testing:** 32 core unit tests covering the lexer, parser, and transpiler, including robust verification of memory management, matching, and indexing.

## 4. Maintenance
- **Reflexive Documentation:** Maintainers **MUST** update `handoff.md` and `guidelines.md` **AUTOMATICALLY AND REFLEXIVELY** without being asked to ensure repository health.
