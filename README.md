# Qalc

Qalc es un compilador para un lenguaje funcional con tipado estático fuerte,
orientado a **computación cuántica**, escrito como un *workspace* de Rust. Su
objetivo es abstraer la complejidad física de los circuitos cuánticos y ofrecer
un modelo de programación de alto nivel, seguro y verificado en tiempo de
compilación. La salida del compilador es **OpenQASM 3**.

El lenguaje ofrece lambdas, *pattern matching*, tipos lineales, aplicación
parcial y bloques monádicos.

## El lenguaje

Qalc es funcional y de tipado estático con inferencia de tipos Hindley-Milner.
Sus rasgos principales son:

- **Tipos lineales.** Los recursos cuánticos (`qcirc`, `qbool`, `qi16`, `qu8`,
  `qnum`) deben usarse exactamente una vez: no se pueden clonar ni descartar. El
  compilador rechaza fugas (uso 0) y duplicaciones (uso >1), garantizando el
  teorema de no-clonación de forma estática.
- **Sin recursión ni bucles.** La recursión está prohibida para que el *inlining*
  (beta-reducción total) siempre termine. La iteración se expresa con la
  primitiva `repeat`.
- **Separación clásico/cuántico.** Los parámetros y el resultado de `main` deben
  ser tipos clásicos; esto permite traducirlos directamente a los modificadores
  `input`/`output` de OpenQASM 3 y producir **circuitos paramétricos**.
- **Bloques monádicos** (`linear { ... }`) para secuenciar operaciones con
  efectos, y el operador *pipe* `|>` para encadenar transformaciones.
- **Identificadores de operador** con *backticks* (`` `+` ``, `` `==` ``) que
  permiten definir funciones invocables tanto con sintaxis infija como prefija.

### Ejemplo: estado de Bell

```qalc
main :: f64 -> qio<u8>
main theta = qcirc 2 |> at 0 H |> controlled 0 1 X |> rz theta |> measure;
```

`main` toma un ángulo clásico (`f64`) como *input* del circuito, construye un
registro cuántico de 2 cúbits, le aplica una Hadamard sobre el cúbit 0, un CNOT
controlado, una rotación `rz` parametrizada y finalmente lo mide.

En `examples/` hay ejemplos más completos, incluyendo un algoritmo de Grover de
2 bits (`grover_2bit.qalc`) con un sumador ripple-carry y un comparador de
igualdad construidos a partir de puertas reales, además de varios ejemplos que
ilustran errores capturados por el compilador (fuga de recursos, no-clonación,
tipos).

## Arquitectura del compilador

El compilador es un *pipeline* donde cada fase transforma el programa en una
representación más rica:

```
Texto fuente
  → [qalc-parser]                     ConcreteAST     (gramática PEG con Pest)
  → [qalc-analysis/normalizer]        NormalizedAST   (desugaring + simplificación)
  → [qalc-analysis/recursion_checker] NormalizedAST   (rechaza recursión)
  → [qalc-analysis/hm_inferrer]       TypedAST        (inferencia de tipos HM)
  → [qalc-analysis/inliner]           FlatAST         (beta-reduce todas las llamadas no-builtin)
  → [qalc-analysis/linear_checker]    FlatAST         (cada recurso lineal usado una vez)
  → [qalc-codegen]                    OpenQASM 3      (texto)
```

Cada fase implementa el *trait* `CompilerStep`, con tipos de entrada y salida
separados. Todos los nodos del AST son genéricos sobre una implementación de
`ASTMetadata`, lo que permite que los mismos tipos de nodo lleven metadatos
distintos en cada fase (por ejemplo `Span` en el AST concreto/normalizado y
`TypeMetadata` en el AST tipado).

### Estructura del workspace

| Crate | Rol |
|---|---|
| `qalc-utils` | Base: `Source`, `Span`, `Diagnostic`, *trait* `CompilerStep` |
| `qalc-ast` | Todos los tipos de AST: concreto, normalizado, tipado, plano, *scope* |
| `qalc-builtins` | `BuiltinDef` y `standard_builtins()` (aritmética + puertas cuánticas + efectos) |
| `qalc-parser` | Texto → `ConcreteAST` mediante Pest PEG + *parsing* de Pratt |
| `qalc-analysis` | `ConcreteAST → NormalizedAST → TypedAST → FlatAST`, más los *checkers* de recursión y linealidad |
| `qalc-codegen` | `FlatAST → OpenQASM 3` (experimental) |
| `qalc-cli` | Ejecutable de entrada (emite OpenQASM 3 por stdout, volcados JSON intermedios por stderr) |

Otros directorios:

- `examples/` — programas de ejemplo en Qalc.
- `report/` — memoria técnica del proyecto (TFG) en LaTeX.
- `scripts/` — utilidades auxiliares (p. ej. `render_qasm.py`).

## Compilación y uso

```bash
# Compilar todos los crates
cargo build

# Ejecutar el compilador sobre un fichero fuente
cargo run -p qalc-cli -- examples/bell.qalc

# Ejecutar todos los tests
cargo test

# Formatear (indentación de 2 espacios)
cargo fmt
```

El CLI emite el OpenQASM 3 resultante por `stdout` y los volcados intermedios
del AST en JSON por `stderr`.

## Licencia

© Esta obra está bajo una licencia de Creative Commons
Reconocimiento-CompartirIgual 4.0 Internacional.
