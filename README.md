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
- **Transpile Project**: `desert transpile path/to/project` (uses `desert.toml`/`Desert.toml` when present; otherwise falls back to `src/main.ds` then `main.ds`)
- **Transpile Current Project**: `desert transpile`
- **Check**: `desert check input.ds` (runs rustc and translates errors)
- **Check Project**: `desert check path/to/project` (same manifest/fallback entrypoint resolution)
- **Check Current Project**: `desert check`
- **Run**: `desert run input.ds`
- **Run Project**: `desert run path/to/project` (same manifest/fallback entrypoint resolution)
- **Run Current Project**: `desert run`
- **Run With Args**: `desert run input.ds -- arg1 arg2`
- **New Project**: `desert new my_app`
- **Format**: `desert fmt path/to/file_or_dir`
- **Format Current Directory**: `desert fmt`
- **Format Check**: `desert fmt path/to/file_or_dir --check`
- **Doctor**: `desert doctor [path/to/file_or_project]`
- **Graph**: `desert graph path/to/project` (prints resolved import order)
- **Graph Current Project**: `desert graph`

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
