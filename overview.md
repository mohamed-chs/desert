# Desert Language Design Specification (v1.0)

**Desert** is a high-performance systems programming language designed for AI researchers, university students, and engineers who require the memory safety and speed of **Rust** but prefer the ergonomic, indentation-based syntax of **Python**.

Desert is not just a "skin"; it is a **semantic bridge**. It transpiles to idiomatic Rust, leveraging the Rust compiler (`rustc`) for optimization while providing a high-level frontend that manages the "noise" of systems-level punctuation.

---

## 1. Core Philosophy
1.  **Readability over Tradition:** Remove the C-style punctuation (`{}`, `;`, `::`) that creates a high cognitive load.
2.  **Transparent Performance:** Every Desert construct maps to an efficient Rust equivalent. No hidden garbage collection.
3.  **Intentional Safety:** Explicit keywords for Ownership moves and Borrowing to prevent the "mental model mismatch" common in Python-to-Rust transitions.
4.  **AI-Native:** First-class support for tensors, multi-dimensional slicing, and Python-native interop.

---

## 2. Syntax & Structure

### 2.1 Blocks and Indentation
Desert uses meaningful whitespace (4 spaces). A colon `:` introduces a block.
*   **No semicolons.** The last expression in a block is the return value.
*   **Explicit suppression:** If a line should not return a value (e.g., to match `()`), use a trailing `;`.

### 2.2 Variables and Mutability
Desert replaces `let` and `let mut` with a more intent-based distinction:
*   `let x = 10`: Immutable binding (Rust `let x = 10;`).
*   `mut y = 20`: Mutable binding (Rust `let mut y = 20;`).
*   `ref z = x`: Shared reference (Rust `let z = &x;`).
*   `mut ref w = y`: Mutable reference (Rust `let w = &mut y;`).

### 2.3 The "Move" Keyword
To prevent confusion regarding Rust's move semantics, Desert requires an explicit indicator when a value is moved into a new scope or variable if that type does not implement `Copy`.
```python
let data = List[1, 2, 3]
let other = move data  # Visual cue that 'data' is no longer valid
```

---

### 3. Namespaces and Pathing
Desert eliminates the `::` separator.
*   **The Unified Dot:** Use `.` for everything (modules, associated functions, methods).
*   **Resolution Strategy:** The Desert transpiler performs a **Name Resolution Pass**. It analyzes the type of the left-hand side:
    *   `Path.new()` → `Path::new()` (Associated function)
    *   `my_path.exists()` → `my_path.exists()` (Method)
*   **The Turbofish:** Replaced by square brackets.
    *   Rust: `collect::<Vec<i32>>()`
    *   Desert: `.collect[List[i32]]()`

---

## 4. Functionality and Types

### 4.1 Functions and Protocols (Traits)
Functions are declared with `def`. Traits are renamed to **Protocols** to align with modern Python/Swift.
```python
protocol Speak:
    def talk(self) -> Str

struct Dog:
    name: Str

impl Speak for Dog:
    def talk(self) -> Str:
        return f"Woof, my name is {self.name}"
```

### 4.2 Error Handling
Desert embraces Rust’s `Result` and `Option` but simplifies the syntax:
*   **Propagation:** Use `?` as a suffix.
*   **Unsafe Unwrap:** Use `!!` (The "Danger" operator).
```python
def get_config(path: Str) -> Config:
    let file = File.open(path)?
    let content = file.read_to_string()?
    return parse(content)!! # I know this won't fail
```

### 4.3 Generics and Collections
Desert uses square brackets for generics to match Python type hints.
*   `List[T]` → `Vec<T>`
*   `Dict[K, V]` → `HashMap<K, V>`
*   `Str` → `String` (The owned type is the default; `&Str` is a slice).

---

## 5. Memory Management Syntax
Desert makes the borrow checker's requirements readable:
*   `&` is used for sharing (immutable borrow).
*   `~` is used for uniqueness (mutable borrow).
```python
def compute(data: &List[i32], target: ~List[i32]):
    # data is read-only
    # target is uniquely yours to modify
```

---

## 6. AI & Python Interop

### 6.1 Python Glue
Desert provides a `pyimport` block to facilitate direct interaction with the CPython interpreter, allowing Desert to act as the "performance engine" for Python apps.
```python
pyimport:
    from numpy import array as nparray
    from torch import tensor

def run_model(data: List[f32]):
    let t = tensor(data) # Direct Python object handling
```

### 6.2 Matrix Multiplication
Since AI is a target domain, Desert reserves `@` for matrix multiplication/tensor operations, mapping to the `Mul` trait or specific ndarray/nalgebra crates.
*   `let c = a @ b`

### 6.3 Macros
Macros are invoked with a `$` prefix to distinguish them from functions and matrix ops.
*   `$print("Value: {x}")`
*   `$vec[1, 2, 3]`

---

## 7. Tooling: The Mirage Engine
Desert ships with a tool suite designed to hide the "transpilation leak":

1.  **Oasis (Package Manager):** A wrapper around `cargo`. It manages `Desert.toml` and automatically fetches Rust dependencies.
2.  **Mirage (Error Translator):** The most critical component. When `rustc` returns an error, Mirage intercepts the JSON output and maps Rust line numbers and concepts back to Desert code.
    *   *Rust Error:* "Expected &mut Vec<i32>, found Vec<i32>"
    *   *Mirage Translation:* "Line 12: You passed a List, but the function requires a unique reference (`~List`). Try passing `~my_list`."
3.  **Sandblast (Formatter):** An opinionated formatter that enforces 4-space indentation and block consistency.

---

## 8. Pragmatic Standards
Desert is "Batteries Included." By default, every Desert project has access to a curated set of **Desert-Core** modules, which are high-level, async-first wrappers around:
*   `tokio` (Async runtime)
*   `serde` (Serialization)
*   `ndarray` (Numerical computing)
*   `reqwest` (Networking)

**Example of a complete Desert file (`main.ds`):**
```python
import desert_core.math as math

def main():
    mut results = List[]
    let inputs = [10, 20, 30]

    for val in inputs:
        if val > 15:
            results.push(math.power(val, 2))
    
    $print("Calculated: {results}")
```

---

## 9. Conclusion
Desert acknowledges that the difficulty of Rust is 20% syntax and 80% mental model. By fixing the 20%, it allows developers to focus entirely on the 80% (Ownership). It provides a high-level "safe haven" for Python developers to enter the world of systems programming without the syntactic friction of the 1970s.
