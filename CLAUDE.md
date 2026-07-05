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
  → [qalc-parser]                    ConcreteAST     (Pest PEG grammar)
  → [qalc-analysis/normalizer]       NormalizedAST   (desugaring + simplification)
  → [qalc-analysis/recursion_checker] NormalizedAST  (rejects recursion so inlining terminates)
  → [qalc-analysis/hm_inferrer]      TypedAST        (HM type inference)
  → [qalc-analysis/inliner]          FlatAST         (beta-reduces all non-builtin calls)
  → [qalc-analysis/linear_checker]   FlatAST         (each linear resource used exactly once)
  → [qalc-codegen]                   OpenQASM 3 text (String)
```

### Crate dependency order (bottom → top)

| Crate | Role |
|---|---|
| `qalc-utils` | Foundation: `Source`, `Span`, `Diagnostic`, `CompilerStep` trait |
| `qalc-ast` | All AST types: concrete, normalized, typed, flat, scope |
| `qalc-builtins` | `BuiltinDef` (name + `TypeWithUnknow`) and `standard_builtins()` (arithmetic + quantum gates + effects) |
| `qalc-parser` | Text → `ConcreteAST` via Pest PEG + Pratt parsing |
| `qalc-analysis` | `ConcreteAST → NormalizedAST → TypedAST → FlatAST`, plus recursion and linear-use checks |
| `qalc-codegen` | `FlatAST → OpenQASM 3` via the `Backend`/`Transpilable<B>` traits (experimental, demo-grade) |
| `qalc-cli` | Executable entry point (emits OpenQASM 3 to stdout, intermediate JSON dumps to stderr) |

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

`crates/qalc-analysis/src/hm_inferrer/` implements HM inference end to end (`define builtins → infer → collect constraints → unify → finalize`). Type declarations are rejected as outside the MVP. Scope (`crates/qalc-ast/src/scope.rs`) supports `define`/`resolve`; type-discriminated overloading (`resolve_call`) is still unimplemented, so keep built-ins single-signature.

### Linear types and quantum backend

`qalc_ast::typed_ast::is_linear_builtin(id)` is the single source of truth for which built-in type ids are linear (`qcirc`, `qbool`, `qi16`, `qu8`, `qnum`). It is consulted wherever a `Builtin { linear }` is constructed (`qalc-builtins::ty`, `hm_inferrer::type_expr_to_infer_type`), so linearity travels with the type and `Type::is_linear()` drives the `LinearChecker`.

The `LinearChecker` (`crates/qalc-analysis/src/linear_checker/`) runs on the `FlatAST` after inlining: it counts syntactic occurrences of each linear-typed binding/param — `0` is a leak, `>1` a duplication. Multiple occurrences of the *same* identifier as direct arguments of *one* application (e.g. `crossToffoli tc 0 tc 1 rc 0`, addressing two different qubits of the same register) are combined via max, not sum — they're one atomic operation, not independent uses that could diverge. Genuine duplication (the same resource read into two separate branches, e.g. tuple elements) is still summed and caught.

**`main`'s params and result must be classical types.** `Type::is_classical()` (in `qalc-ast::typed_ast`, next to `is_linear_builtin`) checks a type isn't linear anywhere inside it (recursing through tuples and unwrapping the `qio` effect wrapper). The `Inliner` (`crates/qalc-analysis/src/inliner/mod.rs`) enforces this on every peeled `FlatParam` and on the result type right after peeling `main`'s arrow type, erroring on the annotation's span otherwise. This is what lets `qalc-codegen` translate `main`'s signature directly to OpenQASM 3 `input`/`output` modifiers, producing parametric circuits (the same program can run with different classical inputs without recompiling).

Since quantum values can no longer enter or leave `main`, they are constructed *inside* the body via built-ins named after the type they build: `qcirc :: i32 -> qcirc` (arg = register size), `qi16 :: i16 -> qi16`, `qu8 :: u8 -> qu8` (always 2 qubits — sized for the 2-bit Grover example, not a real 8-bit register), and `qbool :: i32 -> qbool` (arg = initial classical value; `qbool` takes `i32` because Qalc has no boolean literal syntax yet). Codegen requires that argument to be a compile-time-constant literal (`QasmValue::Int`) — it fixes the register width or the initialization gates — and errors otherwise (see `qcirc_with_non_constant_size_is_rejected` in `qalc-codegen/tests/codegen.rs`). Each constructor call allocates its own auto-named register (`q0`, `q1`, ...), so a program can hold multiple independent quantum resources for free.

**Multi-register gates and generic iteration.** `toffoli`/`controlled` only take one `qcirc` value with indices local to it, so they can't express a gate whose control and target live in different registers — needed for multi-operand arithmetic (a ripple-carry adder over separate `a`/`b`/`sum` registers). `crossControlled`/`crossToffoli` (in `qalc-builtins`) take each qubit as its own `(qcirc, i32)` pair instead, letting codegen emit `cx`/`ccx` across distinct declared registers (valid OpenQASM 3). Qalc has no recursion (`RecursionChecker` forbids it) and no loop/map/fold primitive, so `repeat :: i32 -> (qcirc -> qcirc) -> qcirc -> qcirc` is the only way to give a function like `grover` a genuine iteration-count parameter: its lambda argument survives inlining as a residual `Lambda` (since it's passed to a builtin, not a plain function — see `crates/qalc-analysis/src/inliner/README.md` §6), and `Emitter::eval_repeat` in `qalc-codegen/src/lib.rs` intercepts it *before* generic argument evaluation (which would otherwise reject a bare `Lambda`) and evaluates its body `n` times in sequence, `n` required to be a compile-time constant.

**Qu8/qbool <-> qcirc conversions** (`qcircOfQu8`, `qu8OfQcirc`, `qcircOfQbool`, `qboolOfQcirc`) are identity casts — same `QasmValue` at runtime — that let a function's *signature* stay meaningful (`` `+` :: qu8 -> qu8 -> (qu8, qu8, qu8) ``) while its *body* drops to `qcirc` to apply low-level gates. See `examples/grover_2bit.qalc` for the full pattern: a 2-bit ripple-carry adder and equality comparator built from real `crossControlled`/`crossToffoli` gates (not stubs), composed into an oracle and a generic `grover`.

**Backtick operator identifiers as declarable functions.** Naming a user function `` `+` ``/`` `==` `` (via the backtick `operator_identifier` syntax) works for both declaring *and* calling it, and since `+`/`==` desugar to literal identifiers `"+"`/`"=="` (`normalizer/desugar/binary.rs`), plain infix syntax (`a + b`) resolves to the same user-defined function. This required a parser fix: `crates/qalc-parser/src/declarations.rs` used to stringify the whole `operator_identifier` token (backticks included) for a declaration's name, while expression references correctly stripped them — so a declared `` `+` `` was invisible to any reference to `+`. Fixed via a shared `parse_identifier_token` helper.

`qalc-codegen` walks the `FlatAST` with an effect-evaluation model (circuit-typed values are aliases of a physical register). Basic gates are pure emitters in `registry.rs`; built-ins needing emission context (`qcirc`/`qi16`/`qbool` construction, `measure`/`measureBool`, `qPowConst`, `qEqConst`, `uncompute` — which allocate qreg/ancilla/classical registers) are handled directly in the `Emitter`. The `Emitter::assemble` step promotes a measurement's classical register to an OpenQASM `output` if it ends up being (part of) `main`'s result; otherwise it declares a synthetic `outN` and assigns it. See `examples/bell.qalc` (parametric: takes a classical rotation angle as `input`) and `examples/grover_oracle.qalc`.

### Grammar

`crates/qalc-parser/src/grammar.pest` is the single source of truth for syntax. The Pratt parser in `expression/mod.rs` handles operator precedence. When adding new syntax, change the `.pest` file first, then update the parser, then update the AST types.

## Code Conventions

- Indentation: 2 spaces (enforced by `rustfmt.toml`)
- Error handling: always return `CompilerResult`/`Err(Diagnostic::error(...))`, never panic in compiler logic
- New phases follow the `CompilerStep` pattern — input and output are separate AST types
- Test helpers in `crates/qalc-parser/tests/` use a shared pattern: wrap expression in a minimal function declaration `f :: num f = <expr>;`, parse it, then destructure the body
