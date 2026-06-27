# MiniLisp-RS

A lightweight Mini Lisp interpreter written in Rust, with both:

- a command-line REPL / file runner
- a desktop GUI built with `eframe/egui`

This project is suitable for learning interpreter structure, parsing, evaluation, and simple GUI integration in Rust.

## For Users

### What it can do

MiniLisp-RS currently supports:

- running Lisp expressions interactively in a terminal
- running Lisp code from a `.scm` file
- using a GUI window for code input and output display
- common Lisp-style features such as:
  - arithmetic and comparison
  - `define`, `lambda`, `if`, `begin`, `cond`, `let`
  - lists, pairs, quoting, quasiquoting
  - built-in procedures like `map`, `filter`, `reduce`, `car`, `cdr`, `cons`

### Run the CLI version

Start the REPL:

```bash
cargo run
```

Run a Scheme/Lisp file:

```bash
cargo run -- sort_test1.scm
```

On Windows, press `Ctrl+Z` and then Enter to exit terminal input.

### Run the GUI version

Start the GUI:

```bash
cargo run --bin gui
```

Build a release executable:

```bash
cargo build --release --bin gui
```

The built executable will be at:

```text
target/release/gui.exe
```

You can double-click `gui.exe` on Windows to open the GUI directly.

### GUI usage

In the GUI:

- left panel: input Lisp code
- right panel: view output and evaluation results
- `Run`: execute the current input
- `Clear Output`: clear the output panel
- `Reset Env`: reset interpreter state
- `Ctrl+Enter`: run the current input quickly

The input editor also supports:

- syntax highlighting
- current-line highlighting
- bracket matching

### Example code

You can paste the following examples into the CLI REPL or the GUI input panel.

Basic arithmetic:

```lisp
(+ 1 2 3 4)
(* 6 7)
(/ 7 2)
```

Variables and expressions:

```lisp
(define x 10)
(define y 32)
(+ x y)
```

Function definition:

```lisp
(define (square x) (* x x))
(square 12)
```

Conditional logic:

```lisp
(define n -5)
(if (> n 0) "positive" "not positive")
```

Lists:

```lisp
(list 1 2 3 4)
(car '(10 20 30))
(cdr '(10 20 30))
(append '(1 2) '(3 4))
```

Lambda:

```lisp
((lambda (x y) (+ x y)) 3 4)
```

Mapping and filtering:

```lisp
(map (lambda (x) (* x x)) '(1 2 3 4))
(filter (lambda (x) (> x 2)) '(1 2 3 4))
```

Recursive-style helper example:

```lisp
(define (double x) (+ x x))
(map double '(1 2 3 4))
```

## For Developers

### Common development commands

Check the project:

```bash
cargo check
```

Check the GUI only:

```bash
cargo check --bin gui
```

Run tests:

```bash
cargo test
```

Build all targets:

```bash
cargo build
```

### Project structure

Core interpreter:

```text
src/lib.rs           Public library entry and execute_source
src/main.rs          CLI entry: REPL and file runner
src/token.rs         Token definitions
src/tokenizer.rs     Lexer / tokenizer
src/parser.rs        Parser from tokens to Lisp values
src/value.rs         Runtime value representation
src/eval_env.rs      Evaluation environment and evaluator
src/form.rs          Special forms
src/builtins.rs      Built-in procedures
src/output.rs        Shared output abstraction for CLI and GUI
src/error.rs         Error types
```

GUI:

```text
src/bin/gui.rs           GUI entry, layout, toolbar, interaction flow
src/bin/gui/theme.rs     GUI theme and egui style application
src/bin/gui/editor.rs    Editor highlighting and editor decoration logic
```

### How the GUI code is organized

The GUI was intentionally split by responsibility:

- `gui.rs`
  - owns app state
  - wires buttons and panels
  - runs interpreter code
- `theme.rs`
  - stores visual settings like colors, spacing, sizes, fonts
  - applies theme values to egui
- `editor.rs`
  - handles syntax highlighting
  - handles current-line and bracket-match decorations

If you want to:

- change colors / spacing / font sizes: edit `src/bin/gui/theme.rs`
- change editor highlighting rules: edit `src/bin/gui/editor.rs`
- change window layout or button behavior: edit `src/bin/gui.rs`

### Recommended development workflow

1. Make core language changes in `src/lib.rs` and related interpreter modules.
2. Validate behavior with `cargo test`.
3. If GUI behavior is affected, run:

```bash
cargo check --bin gui
```

4. If you changed layout or styling, run the GUI locally:

```bash
cargo run --bin gui
```

### Notes for extending the interpreter

When adding a new language feature, the usual flow is:

1. Add or update tokenization if needed in `tokenizer.rs`
2. Update parsing in `parser.rs`
3. Add evaluation logic in `form.rs`, `builtins.rs`, or `eval_env.rs`
4. Add tests in the existing test modules
5. If the feature affects displayed output, verify both CLI and GUI behavior

## License

See [LICENSE](LICENSE).
