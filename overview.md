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
- Unified dot resolution with scoped type/value tracking:
  - static call: `Type.new()` -> `Type::new()`
  - method call: `value.method()` -> `value.method()`

## CLI

- `desert transpile <file.ds> [-o file.rs]`
- `desert check <file.ds>`
- `desert transpile <project_dir>` with `desert.toml`/`Desert.toml` (`[package].entry`, default `src/main.ds`)
- `desert check <project_dir>` with the same project entry resolution
- `desert graph <project_dir>` to print resolved import load order

Project mode resolves top-level `import` statements recursively, loads imported files before importers, and rejects import cycles.

`check` transpiles to Rust, runs `rustc --emit=metadata --error-format=json`, and maps diagnostics back to Desert source lines.

## Current Limits

- No per-file span diagnostics yet in project mode (imports compile as a combined source stream today).
- Resolver is scoped symbol tracking, not full type inference.
- `pyimport` is preserved as Rust comments.
- Source map is line-based.
- `@` lowering is currently specialized to float vector/matrix helpers.

## Direction

1. Expand semantic pre-checking before Rust emission.
2. Deepen Mirage diagnostics with targeted fix hints.
3. Stabilize a narrow core language before adding new syntax.
