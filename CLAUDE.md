# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Qalc

Qalc is a compiler for a functional language with strong static typing, written as a Rust workspace. The long-term goal is quantum computing support. The language features lambdas, pattern matching, linear types, partial application, and monadic blocks.

## Build and Test Commands

```bash
# Build all crates
cargo build

# Run the CLI on a source file
cargo run -p qalc-cli -- path/to/file.qalc

# Run all tests
cargo test

# Run tests for a single crate
cargo test -p qalc-parser

# Run a single test by name
cargo test -p qalc-parser -- test_name

# Format (2-space indent, no hard tabs)
cargo fmt
```

## Architecture

The compiler is a pipeline where each phase transforms the program into a richer representation:

```
Source text
  → [qalc-parser]   ConcreteAST      (Pest PEG grammar)
  → [qalc-analysis/normalizer]   NormalizedAST   (desugaring + simplification)
  → [qalc-analysis/hm_inferrer]  TypedAST        (HM type inference — mostly todo!())
```

### Crate dependency order (bottom → top)

| Crate | Role |
|---|---|
| `qalc-utils` | Foundation: `Source`, `Span`, `Diagnostic`, `CompilerStep` trait |
| `qalc-ast` | All AST types: concrete, normalized, typed, scope |
| `qalc-parser` | Text → `ConcreteAST` via Pest PEG + Pratt parsing |
| `qalc-analysis` | `ConcreteAST → NormalizedAST` (done) and `NormalizedAST → TypedAST` (in progress) |
| `qalc-cli` | Executable entry point |

### Parameterized AST pattern

All AST nodes are generic over an `ASTMetadata` impl. This lets the same node types hold different metadata across phases:

- **`Spanned`** — all meta fields are `Span` (used in concrete + normalized AST)
- **`Typed`** — meta fields are `TypeMetadata { span, ttype }` (used in typed AST)

Adding a new compiler phase means defining a new `ASTMetadata` impl and a new `CompilerStep` impl; no changes to the shared node types are needed.

### CompilerStep trait

Every phase implements:
```rust
pub trait CompilerStep {
  type Input;
  type Output;
  fn transform(&self, input: Self::Input, diagnostics: &mut Vec<Diagnostic>) -> CompilerResult<Self::Output>;
}
```

Errors use `CompilerResult<T> = Result<T, Diagnostic>` and always carry a `Span` for precise source-location reporting via `ariadne`.

### Type system in qalc-ast

`TypeWithUnknow` is used during inference — it can hold unresolved `Unknow(usize)` type variables. After unification it converts to `Type` (the fully-resolved form). `TypeConstraint(a, b)` asserts that two `TypeWithUnknow` must unify.

### HM type inferrer status

`crates/qalc-analysis/src/hm_inferrer/` contains the skeleton:
- `mod.rs` — `HMInferrer` struct, `transform()` is `todo!()`
- `infer.rs` — `infer_expr()` handles literals, identifiers, tuples, slices, if-else; lambdas, let-blocks, and transforms are still `todo!()`

Scope (`crates/qalc-ast/src/scope.rs`) supports `define`/`resolve` for values but `define_func`/`resolve_call` (for function overloading) are `todo!()`.

### Grammar

`crates/qalc-parser/src/grammar.pest` is the single source of truth for syntax. The Pratt parser in `expression/mod.rs` handles operator precedence. When adding new syntax, change the `.pest` file first, then update the parser, then update the AST types.

## Code Conventions

- Indentation: 2 spaces (enforced by `rustfmt.toml`)
- Error handling: always return `CompilerResult`/`Err(Diagnostic::error(...))`, never panic in compiler logic
- New phases follow the `CompilerStep` pattern — input and output are separate AST types
- Test helpers in `crates/qalc-parser/tests/` use a shared pattern: wrap expression in a minimal function declaration `f :: num f = <expr>;`, parse it, then destructure the body
