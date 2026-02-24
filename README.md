# Desert

Desert is an indentation-based language that transpiles to Rust, combining Python-like syntax with Rust's performance and safety model.

## Features

- **Syntax**: Indentation-driven blocks (`if`, `def`, `struct`, `impl`, etc.).
- **Bindings**: Canonical declarations with `let` and `mut`.
- **Ownership**: Explicit `move`, `&` (shared), and `~` (unique) expression markers.
- **Projects**: Directory mode with manifest entrypoint and recursive `import` loading.
- **Generics**: Bracketed syntax (e.g., `List[T]`).
- **Math**: Native `@` operator for matrix multiplication.
- **Diagnostics**: Mirage layer translates Rust compiler errors back to Desert terms.

## Usage

### Installation
Requires [Rust and Cargo](https://rustup.rs/).

```bash
cargo build --release
```

### Commands
- **Transpile**: `desert transpile input.ds -o output.rs`
- **Transpile Project**: `desert transpile path/to/project` (expects `desert.toml` or `Desert.toml`, default entry `src/main.ds`)
- **Check**: `desert check input.ds` (runs rustc and translates errors)
- **Check Project**: `desert check path/to/project` (same manifest/entrypoint resolution)
- **Run**: `desert run input.ds`
- **Run Project**: `desert run path/to/project`
- **Run With Args**: `desert run input.ds -- arg1 arg2`
- **Graph**: `desert graph path/to/project` (prints resolved import order)

Project source files can import other files using `import "relative/path.ds"` or dotted paths like `import util.math` (resolved relative to the importing file, `.ds` extension implied).

## Example

```python
def main():
    mut count = 10
    let moved = move count
    $print("Value: {moved}")
```

## Architecture

- **Lexer**: Logos-based with indentation tracking.
- **Parser**: Recursive-descent with span preservation.
- **Transpiler**: Generates Rust and source maps.
- **Mirage**: Intercepts and rewrites `rustc` diagnostics.

## Testing

Run example-based integration tests:
```bash
cargo test --test check_examples
```
