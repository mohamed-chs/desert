# Desert Language Overview

Desert is an indentation-based systems language frontend that transpiles to Rust.

The design priority is explicit semantics with predictable lowering. Syntax that duplicates meaning is removed aggressively, even when breaking existing code.

## Current Language Surface

- Blocks: `if`, `for`, `def`, `struct`, `protocol`, `impl`, `match`
- Bindings: `let` and `mut`
- Ownership and borrows:
  - `move place` lowers to `std::mem::take(&mut place)`
  - `&expr` shared borrow
  - `~expr` mutable borrow
  - Borrow bindings are expression-only (`let r = &x`, `let r = ~x`); statement-level `ref`/`mut ref` syntax was removed
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

`check` transpiles to Rust, runs `rustc --emit=metadata --error-format=json`, and maps diagnostics back to Desert source lines.

## Current Limits

- Single-file workflow only.
- Resolver is scoped symbol tracking, not full type inference.
- `pyimport` is preserved as Rust comments.
- Source map is line-based.
- `@` lowering is currently specialized to float vector/matrix helpers.

## Direction

1. Expand semantic pre-checking before Rust emission.
2. Deepen Mirage diagnostics with targeted fix hints.
3. Stabilize a narrow core language before adding new syntax.
