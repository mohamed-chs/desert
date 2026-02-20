# Desert Language Overview

Desert is an indentation-based language that transpiles to Rust.

The goal is straightforward: keep Rust's performance and ownership model, but make the surface syntax feel closer to Python. Desert is currently a parser/transpiler prototype with a small CLI (`transpile`, `check`) and a Mirage-style error translation layer.

## What Works Right Now

- Indentation-driven blocks (`if`, `for`, `def`, `struct`, `protocol`, `impl`, `match`)
- Variable forms: `let`, `mut`, `ref`, `mut ref`
- Expressions and operators:
  - arithmetic: `+`, `-`, `*`, `/`
  - comparisons: `==`, `!=`, `<`, `<=`, `>`, `>=`
  - assignment in expressions (`=`)
  - matrix operator token `@` (currently transpiles as `*`)
  - indexing: `expr[index]`
  - error operators: `?` and `!!`
  - ownership/borrow markers: `move`, `&`, `~`
- Macro calls via `$name(...)` (`$print(...)` maps to `println!`)
- Generic syntax with brackets:
  - types: `List[i32]` -> `Vec<i32>`
  - generic calls: `obj.method[T](arg)` -> `obj.method::<T>(arg)`
- Unified dot resolution:
  - `Type.new()` -> `Type::new()`
  - `value.method()` -> `value.method()`

## CLI

- `desert transpile <file.ds> [-o file.rs]`
- `desert check <file.ds>`

`check` transpiles the Desert file, runs `rustc --error-format=json`, and translates diagnostics back to Desert terms where possible.

## Current Limits

This is still an experimental compiler frontend. A few important constraints:

- No package/project system yet (single-file workflow).
- Type and name resolution are intentionally simple.
- `pyimport` blocks are preserved as structured comments in Rust output.
- Source maps are line-based (not full byte-accurate remapping).

## Direction

The most practical next milestones are:

1. Better diagnostics and span mapping.
2. Stronger name/type resolution.
3. Integration tests that compile example `.ds` files end-to-end.
4. A small, explicit Desert standard library surface.
