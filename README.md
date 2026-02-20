# Desert

Desert is an indentation-based language that transpiles to Rust, combining Python-like syntax with Rust's performance and safety model.

## Features

- **Syntax**: Indentation-driven blocks (`if`, `def`, `struct`, `impl`, etc.).
- **Ownership**: Explicit `move`, `&` (shared), and `~` (unique) markers.
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
- **Check**: `desert check input.ds` (runs rustc and translates errors)

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

## License

MIT License (see project root).
