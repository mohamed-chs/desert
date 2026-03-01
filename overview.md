# Desert Language Overview

Desert is an indentation-based systems language frontend that transpiles to Rust.

The design priority is explicit semantics with predictable lowering. Syntax that duplicates meaning is removed aggressively, even when breaking existing code.

## Current Language Surface

- Blocks: `if`, `for`, `def`, `struct`, `protocol`, `impl`, `match`
- Imports: `import "relative/path.ds"` and `import dotted.module` (resolved relative to the importing file, `.ds` default extension)
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
- `impl`/`protocol` method declarations:
  - Duplicate method names within the same `impl` or `protocol` block are rejected during semantic validation
  - `impl` and `protocol` bodies only allow `def` declarations
- Unified dot resolution with scoped type/value tracking:
  - static call: `Type.new()` -> `Type::new()`
  - method call: `value.method()` -> `value.method()`

## CLI

- `desert transpile <file.ds> [-o file.rs]`
- `desert check <file.ds>`
- `desert run <file.ds> [-- args...]`
- `desert transpile <project_dir>` with `desert.toml`/`Desert.toml` (`[package].entry`, default `src/main.ds`)
- `desert check <project_dir>` with the same project entry resolution
- `desert run <project_dir> [-- args...]` with the same project entry resolution
- `desert new <path> [--force]` to scaffold `desert.toml` and `src/main.ds`
- `desert fmt <file_or_dir> [--check]` to apply/enforce canonical Desert formatting
- `desert doctor [file_or_project]` to preflight rustc availability and source/project parse+semantic health
- `desert graph <project_dir>` to print resolved import load order

Project mode resolves top-level `import` statements recursively, loads imported files before importers, and rejects import cycles.

`check` transpiles to Rust, runs `rustc --emit=metadata --error-format=json`, and maps diagnostics back to Desert source file+line locations.

`run` transpiles and compiles to a temp executable, reports translated compile diagnostics on failure, and executes the program directly.

## Current Limits

- Rust diagnostics are mapped at file+line granularity; column mapping is not yet source-accurate.
- Resolver is scoped symbol tracking, not full type inference.
- `pyimport` is preserved as Rust comments.
- `@` lowering is currently specialized to float vector/matrix helpers.

## Direction

1. Expand semantic pre-checking before Rust emission.
2. Deepen Mirage diagnostics with targeted fix hints.
3. Stabilize a narrow core language before adding new syntax.
