# Desert Language Overview

Desert is an indentation-based systems language frontend that transpiles to Rust.

The design priority is explicit semantics with predictable lowering. Syntax that duplicates meaning is removed aggressively, even when breaking existing code.

## Current Language Surface

- Blocks: `if`, `for`, `def`, `struct`, `protocol`, `impl`, `match`
- Imports: `import "relative/path.ds"` and `import dotted.module` (resolved relative to the importing file, `.ds` default extension), plus Rust `use` passthrough via `import rust.std.cmp.max`, `import rust.std.collections.HashMap as Map`, `from rust.std.cmp import max as maximum, min`, or `import "rust:std::cmp::max"` (`std`/`core`/`alloc` roots)
  - `from ... import ...` now rejects duplicate imported items and duplicate introduced local names within a single statement
- Bindings: `let` and `mut`
- Ownership and borrows:
  - `move place` lowers to `std::mem::take(&mut place)`
  - `&expr` shared borrow
  - `~expr` mutable borrow
  - `move`/`~` pre-checks treat `x` as requiring a `mut` binding, while `obj.field`/`items[i]` allow unique-reference write-through roots
  - Borrow bindings are expression-only (`let r = &x`, `let r = ~x`); statement-level `ref`/`mut ref` syntax was removed
  - Assignment pre-checks require place-form left-hand sides (`x`, `obj.field`, `items[i]`) and enforce mutable/write-through roots before Rust emission
- Operators:
  - arithmetic: `+`, `-`, `*`, `/`
  - comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - assignment expression: `=`
  - matrix operator: `@` lowering via generated `desert_matmul(...)` helpers
  - indexing: `expr[index]`
  - error operators: `?`, `!!`
- Macros: `$name(...)` (`$print(...)` maps to `println!`)
- Generics:
  - type forms: `List[i32]` -> `Vec<i32>`
  - generic calls: `obj.method[T](arg)` -> `obj.method::<T>(arg)`
- Struct constructor calls:
  - `Type(...)` is validated before Rust emission: named args must be `field = value`, fields must exist and be unique, positional arity must fit, and required fields cannot be omitted
- Top-level declarations:
  - Duplicate top-level `def`/`struct`/`protocol` names are rejected during semantic validation before Rust emission
  - Top-level names must also be unique across declaration kinds (`def`/`struct`/`protocol`)
- Local bindings:
  - `let`/`mut` redeclaration of the same name in a single block scope is rejected during semantic validation
  - Local `def` names must also be unique within a single block scope and cannot collide with already-declared local names
  - Calls to declared `def` names are arity-checked during semantic validation (including forward local defs and generic-call syntax)
  - Unresolved identifiers now fail during semantic validation (`unknown identifier ...`) instead of deferring to rustc, while preserving built-in enum/bool value symbols and declared type receivers
- Match arm patterns:
  - Pattern binders (for example `Some(node)`) are introduced into the corresponding arm scope before body validation
- `impl`/`protocol` method declarations:
  - Duplicate method names within the same `impl` or `protocol` block are rejected during semantic validation
  - `impl` and `protocol` bodies only allow `def` declarations
  - `impl` targets must be declared `struct` types, and `impl Protocol for Type` requires a declared `protocol`
  - Protocol impls must match declared protocol method sets and signatures (no missing/extra methods, and compatible param/return signatures)
- Unified dot resolution with scoped type/value tracking:
  - static call: `Type.new()` -> `Type::new()`
  - method call: `value.method()` -> `value.method()`

## CLI

- `desert transpile <file.ds> [-o file.rs]`
- `desert transpile` defaults to current directory
- `desert check <file.ds> [--stage syntax|semantic|rust]`
- `desert check` defaults to current directory
- `desert run <file.ds> [-- args...]`
- `desert run` defaults to current directory
- `desert transpile <project_dir>` with `desert.toml`/`Desert.toml` (`[package].entry`, default `src/main.ds`), or fallback entry resolution (`src/main.ds`, then `main.ds`) when no manifest exists
- `desert check <project_dir> [--stage syntax|semantic|rust]` with the same project entry resolution
- `desert run <project_dir> [-- args...]` with the same project entry resolution
- `desert new <path> [--force]` to scaffold `desert.toml` and `src/main.ds`
- `desert fmt <file_or_dir> [--check]` to apply/enforce canonical Desert formatting
- `desert fmt` defaults to current directory
- `desert doctor [file_or_project]` to preflight rustc availability and source/project parse+semantic health
- `desert graph <project_dir>` to print resolved import load order
- `desert graph` defaults to current directory

Project mode resolves top-level `import` statements recursively, loads imported files before importers, and rejects import cycles.

`check` transpiles to Rust, runs `rustc --emit=metadata --error-format=json`, and maps diagnostics back to Desert source file+line locations.

`run` transpiles and compiles to a temp executable, reports translated compile diagnostics on failure, and executes the program directly.

## Current Limits

- Rust diagnostics are mapped at file+line+statement-start-column granularity; token-accurate column mapping is not yet implemented.
- Resolver is scoped symbol tracking, not full type inference.
- `pyimport` is preserved as Rust comments.
- `@` lowering is currently specialized to float vector/matrix helpers.

## Direction

1. Expand semantic pre-checking before Rust emission.
2. Deepen Mirage diagnostics with targeted fix hints.
3. Stabilize a narrow core language before adding new syntax.
