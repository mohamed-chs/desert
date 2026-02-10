This implementation plan outlines the roadmap for moving **Desert** from a design specification to a production-ready toolchain. The plan is divided into five strategic phases, focusing on the hardest technical challenges first (Name Resolution and Error Mapping).

---

## Phase 1: The Lexical Foundation
**Goal:** Build a robust, indentation-aware parser that can handle the transition from curly-brace semantics to whitespace semantics.

*   **1.1 The Indentation Lexer:**
    *   Implement a lexer that maintains a stack of indentation levels.
    *   Emit virtual `INDENT` and `DEDENT` tokens (similar to Python’s lexer).
*   **1.2 The AST Definition:**
    *   Define a Desert AST that mirrors Rust’s `syn` crate but includes Desert-specific nodes (e.g., `MatrixAttr`, `PythonImport`).
*   **1.3 The "Pass-Through" Parser:**
    *   Build a parser (using a parser generator like `LALRPOP` or a handwritten recursive descent parser) that transforms `.ds` code into a basic AST.
    *   **Milestone:** Transpile a "Hello World" and a basic `factorial` function from Desert to Rust.

---

## Phase 2: The Semantic Bridge
**Goal:** Resolve the "Unified Dot" and "Generics" problem. This is the most technically complex phase.

*   **2.1 The Symbol Resolver:**
    *   Integrate with `cargo metadata` to understand the dependency tree.
    *   Implement a partial name resolver. To distinguish between `Path.new()` (`::`) and `path.exists()` (`.`), Desert will:
        1.  Check if the prefix is a known Type or Module in the current scope.
        2.  If yes, transpile to `::`.
        3.  If no, assume it is an instance and transpile to `.`.
*   **2.2 Generic Mapping:**
    *   Transform `List[i32]` into `Vec<i32>`.
    *   Handle the "Turbofish" logic: detect if a generic is being called in an expression context and inject the `::<>` syntax into the generated Rust.
*   **2.3 Move Tracking:**
    *   Enforce the `move` keyword requirement for non-`Copy` types by inspecting the trait implementations of used types (via `rustc` metadata).

---

## Phase 3: The Mirage Error System
**Goal:** Ensure that users never have to look at generated Rust code to fix a Desert error.

*   **3.1 Source Mapping:**
    *   Generate a source map during transpilation that links every byte of the `.rs` file back to the `.ds` file.
*   **3.2 The Mirage Proxy:**
    *   Build a wrapper around `cargo check`.
    *   Intercept the JSON output of Rust errors.
    *   **Mapping Logic:** Rewrite the error message. Replace Rust symbols (`Vec`, `&mut`, `::`) with Desert symbols (`List`, `~`, `.`).
    *   **Milestone:** A user makes a borrow-checker error in Desert, and the CLI points to the correct line in the `.ds` file with a Desert-friendly explanation.

---

## Phase 4: Interop and AI Integration
**Goal:** Implement the "AI-Native" features that make Desert better for researchers than pure Rust.

*   **4.1 PyO3 Integration:**
    *   The `pyimport` block will automatically generate the boilerplate needed by `PyO3` to initialize a Python interpreter and handle GIL locking.
*   **4.2 Desert-Core (The "Oasis" Library):**
    *   Develop the standard library.
    *   Write wrappers for `ndarray` to support the `@` operator.
    *   Implement the `$vec` and `$print` macro transpilation.
*   **4.3 Oasis Package Manager:**
    *   Create the `Oasis` CLI. It should manage a hidden Rust project in a `.desert/` directory, handling the `Cargo.toml` automatically.

---

## Phase 5: Ecosystem and Polishing
**Goal:** Professional-grade developer experience.

*   **5.1 Language Server Protocol (LSP):**
    *   Build a Desert-LSP that wraps `rust-analyzer`.
    *   Provide syntax highlighting, auto-completion, and "Go to Definition" within `.ds` files.
*   **5.2 Documentation Generator:**
    *   A tool that reads Desert doc-comments (`##`) and generates a searchable website (akin to `rustdoc`).
*   **5.3 University Pilot:**
    *   Release a "Lab Starter Kit" for a systems programming course, including a VS Code extension and a set of AI-focused tutorials.

---

## Technical Stack Recommendation
*   **Implementation Language:** Rust (for performance and library compatibility with `syn` and `rust-analyzer`).
*   **Parsing:** `nom` or `rowan` (for concrete syntax trees and better IDE support).
*   **CLI:** `clap`.
*   **Testing:** `insta` (snapshot testing for transpilation output).

---

## Risk Mitigation
*   **The "Complex Macro" Risk:** Rust macros can be incredibly complex.
    *   *Solution:* Allow a "raw" block `raw_rust:` for edge cases where Desert syntax fails to capture a complex macro.
*   **The "Performance Overhead" Risk:** Transpilation might be slow.
    *   *Solution:* Use an incremental transpilation strategy. Only re-transpile files that have changed, using a file-hash cache.
*   **The "Dependency Hell" Risk:** Rust crates update frequently.
    *   *Solution:* Oasis will pin `desert-core` to specific, tested versions of underlying Rust crates to ensure stability for students.
