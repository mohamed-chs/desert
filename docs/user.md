# Desert User Guide

## What Desert Is

Desert is an indentation-based language that transpiles to Rust.

Use it for a Python-like authoring experience with Rust compilation and diagnostics.

## Install

Prerequisite: Rust + Cargo (`rustup` install).

Build from source:

```bash
cargo build --release
```

Binary path:

```bash
./target/release/desert
```

Optional local alias:

```bash
alias desert="$(pwd)/target/release/desert"
```

## 2-Minute Start

```bash
cargo build --release
./target/release/desert new hello_desert
cd hello_desert
../target/release/desert run
```

Create a single file and run it:

```python
def main():
    $print("hello")
```

```bash
desert run main.ds
```

## Project Structure

Recommended layout:

```text
my_app/
  desert.toml
  src/
    main.ds
    util/
      math.ds
```

Minimal `desert.toml`:

```toml
[package]
name = "my_app"
entry = "src/main.ds"
```

Entrypoint resolution order:

1. `[package].entry` in `desert.toml` or `Desert.toml`
2. `src/main.ds`
3. `main.ds`

## Command Reference

### `desert transpile`

Convert Desert to Rust.

```bash
desert transpile src/main.ds -o main.rs
desert transpile .
```

### `desert check`

Validate code without running it.

```bash
desert check
```

Stage modes:

```bash
desert check --stage syntax
desert check --stage semantic
desert check --stage rust
```

Use `syntax` for parser issues, `semantic` for Desert-level validation, `rust` for full Rust-toolchain-backed checks (via Cargo).

### `desert run`

Compile and execute file or project.

```bash
desert run
desert run src/main.ds
desert run . -- arg1 arg2
```

### `desert fmt`

Apply canonical formatting.

```bash
desert fmt
desert fmt src/main.ds
desert fmt . --check
```

### `desert doctor`

Check environment and input health.

```bash
desert doctor
desert doctor .
```

### `desert graph`

Print resolved import load order.

```bash
desert graph
desert graph .
```

### `desert new`

Scaffold a new project.

```bash
desert new my_app
desert new my_app --force
```

## Imports and Modules

Supported forms:

```python
import "util/math.ds"
import util.math
import rust.std.collections.HashMap as Map
from rust.std.cmp import max as maximum, min
import rust.std.cmp.max
import "rust:std::io::Read"
```

Rules:

- Imports are top-level only.
- Dotted imports resolve relative to the importing file.
- `.ds` extension is implied for dotted forms.
- Import cycles are rejected.
- `import rust...` / `import "rust:..."` lower to Rust `use ...;` and currently support `std`, `core`, and `alloc` roots.
- `from rust... import ...` lowers to grouped Rust `use base::{...};`.

## Language Quick Reference

### Bindings and Mutability

```python
let x = 1
mut y = 2
y = 3
```

### Functions

```python
def add(a: i32, b: i32) -> i32:
    return a + b
```

### Structs and Constructors

```python
struct Point:
    x: f32
    y: f32

let p1 = Point(1.0, 2.0)
let p2 = Point(x = 1.0, y = 2.0)
```

Constructor calls are validated before Rust emission (unknown, missing, duplicate fields fail early).

### Protocols and Impl Blocks

```python
protocol Speak:
    def speak(self) -> String

struct Dog:
    name: String

impl Speak for Dog:
    def speak(self) -> String:
        return self.name
```

### Borrow and Move Forms

```python
mut count = 10
let r1 = &count
let r2 = ~count
let taken = move count
```

Rules:

- `&expr`: shared borrow.
- `~expr`: mutable borrow.
- `move expr`: lowers to `std::mem::take(&mut expr)`.
- `move` and `~` operands must be place expressions (`x`, `obj.field`, `items[i]`).

### Operators

- Arithmetic: `+ - * /`
- Modulo: `%`
- Logic: `and or not`
- Comparison: `== != < <= > >=`
- Assignment: `=`
- Matrix multiply: `@`
- Indexing: `expr[index]`
- Error operators: `?`, `!!`

### Macros

```python
$print("value = {x}")
```

`$print(...)` lowers to `println!(...)`.

### Match

```python
match value:
    Some(node):
        $print("{node}")
    None:
        $print("empty")

### While / Loop Control

```python
mut i = 0
while i < 10:
    i = i + 1
    if i % 2 == 0:
        continue
    if i == 9:
        break
```
```

Pattern binders are scoped to their arm body.

## Diagnostics You Will See

Common semantic errors:

- `unknown identifier ...`: symbol not declared in scope.
- duplicate local binding in same block.
- duplicate top-level declaration names.
- duplicate method names inside one `impl` or `protocol`.
- `import` outside top level.
- `return` outside a `def` body.
- `move`/`~` operand is not a place expression.
- invalid assignment target (left side is not a place).

## Practical Workflow

Normal iteration loop:

1. `desert fmt`
2. `desert check --stage semantic`
3. `desert check`
4. `desert run -- ...`

Before a commit:

1. `cargo test`
2. `for f in examples/*.ds; do desert check "$f"; done`

## Troubleshooting

`desert: command not found`

- Use the full path `./target/release/desert`, or add it to `PATH`.

`rustc`-related failures

- Run `desert doctor`.
- Verify Rust toolchain installation with `rustc --version`.

Import path issues

- Use paths relative to the importing file.
- Keep imports top-level.
- Use `desert graph` to inspect load order.

Formatter changed files unexpectedly

- Run `desert fmt` locally before CI.
- Use `desert fmt . --check` in CI.
