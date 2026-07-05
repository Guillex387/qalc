# Contexto Completo del Compilador Qalc

## Visión General

**Qalc** es un compilador modular escrito en Rust diseñado como un workspace de múltiples crates. El proyecto implementa un compilador funcional con análisis de tipos basado en inferencia Hindley-Milner, normalización de código y soporte para características avanzadas como expresiones lambda, patrones, tipos lineales y operaciones parciales.

### Propósito Principal
Compilar un lenguaje funcional con tipado estático fuerte hacia representaciones intermedias normalizadas y tipadas. El proyecto está en desarrollo activo y aún contiene métodos `todo!()` que necesitan implementación (especialmente el HM type inferrer).

---

## Arquitectura General

El compilador sigue un **pipeline clásico de compilación** con múltiples representaciones del programa:

```
Código Fuente 
    ↓
[Parser] → ConcreteAST
    ↓
[Normalizer] → NormalizedAST
    ↓
[Recursion Checker] → NormalizedAST
    ↓
[HM Type Inferrer] → TypedAST
    ↓
[Inliner] → FlatAST
    ↓
[Linear Checker] → FlatAST
    ↓
[Codegen (qalc-codegen)] → OpenQASM 3
```

Cada fase transforma el código a una representación más refinada, añadiendo información semántica y realizando desugaring de características sintácticas.

---

## Estructura de Crates

El workspace está organizado en 5 crates principales + documentación:

### 1. **qalc-utils** (Fundación)
**Propósito**: Proporcionar tipos y utilidades compartidas por todo el compilador.

**Ubicación**: `crates/qalc-utils/src/`

**Módulos principales**:
- **`source.rs`**: Gestiona la representación del código fuente
  - `Source`: Abstracción de archivo o memoria
  - Métodos para cargar archivos (`Source::file()`) o código en memoria (`Source::memory()`)
  - Integración con la librería `ariadne` para reportes de errores visualmente atractivos
  
- **`span.rs`**: Rastreo de ubicaciones en el código fuente
  - `Span`: Contiene referencia a source + posición de inicio/fin (bytes)
  - Usado para todos los mensajes de error y diagnósticos
  - Método `join()` para combinar spans
  
- **`diagnostic.rs`**: Sistema de diagnósticos del compilador
  - `Diagnostic`: Estructura para representar errores, advertencias y notas
  - `DiagnosticKind`: Enum con tipos (Error, Warning, Note)
  - `CompilerResult<T>`: Alias para `Result<T, Diagnostic>`
  - Rendering integrado con `ariadne` para mostrar errores contextualizados
  
- **`compiler_step.rs`**: Trait base para fases del compilador
  - `CompilerStep`: Trait que define el contrato de cada fase compilación
  - Permite transformar entrada → salida con manejo de diagnósticos
  - Implementado por `Normalizer` y `HMInferrer`

**Dependencias externas**:
- `ariadne 0.6.0`: Librería de reportes de errores con colores
- `derive_more 2.1.1`: Derivación automática de traits adicionales

---

### 2. **qalc-ast** (Representaciones Abstractas)
**Propósito**: Definir todos los tipos AST y sus variaciones según la fase de compilación.

**Ubicación**: `crates/qalc-ast/src/`

**Módulos principales**:

#### **`ast_metadata.rs`**: Sistema de Metadata Paramétrico
Implementa un patrón genérico para adjuntar metadatos (información contextual) a nodos del AST:

- **`ASTMetadata` trait**: Define 4 tipos asociados para metadatos
  - `DeclarationMeta`: Info de declaraciones (tipos, funciones)
  - `ExpressionMeta`: Info de expresiones
  - `PatternMeta`: Info de patrones
  - `TypeMeta`: Info de expresiones de tipo
  
- **`Spanned` struct**: Implementación de `ASTMetadata`
  - Todos los metadatos son de tipo `Span`
  - Usado en fases tempranas (Concrete y Normalized)

- **Ventaja**: Un mismo AST puede contener diferentes metadatos según la fase (Span vs Type Info)

#### **`base_ast/mod.rs`**: Definiciones Base
Tipos genéricos parametrizados sobre un `ASTMetadata` que sirven como base:

- **`Declaration<M>`**: Una declaración (tipo o función)
  - Contiene `DeclarationKind` y metadatos `M::DeclarationMeta`
  
- **`DeclarationKind<M>`**: 
  - `Type`: Declaración de tipo con atributos y flag linear
  - `Function`: Declaración de función con tipo, parámetros y cuerpo
  
- **`Expression<M>`**: Expresión genérica (no used directamente, base para otros AST)
  - Contiene `ExpressionKind` y metadatos
  
- **`Pattern<M>`**: Patrón usado en pattern matching y bindings
  - `Variable`: Vinculación a variable
  - `Tuple`: Patrón de tupla
  - `Wildcard`: Pattern `_` que ignora valor
  
- **`TypeExpression<M>`**: Expresión de tipo
  - `Identifier`: Tipo simple por nombre
  - `IdentifierWithParams`: Tipo genérico (ej: `List Int`)
  - `Slice`: Tipo slice (colección)
  - `Tuple`: Tipo tupla
  - `Transform`: Tipo función (input → output)

- **Otros**: `NumberSuffix`, `FloatSuffix`: Sufijos numéricos (u8, i32, f64, etc.)

#### **`concrete_ast.rs`**: Primera Representación (tras Parser)
Representación directa del código parseado, sin normalización:

- **`ConcreteAST`**: Raíz del AST concreto
  - `declarations: Vec<ConcreteDeclaration>`
  
- **`ConcreteDeclaration`**: Con span (ubicación en código fuente)
  - `ConcreteDeclarationKind::Type` / `::Function`
  
- **`ConcreteExpression`**: Expresión con span
  - **Expresiones de bloque**: `LetBlock`, `IfElseBlock`, `MatchBlock`, `PerformBlock`, `LinearBlock`
  - **Operaciones binarias**: `Binary`, `PartialLeftBinary`, `PartialRightBinary`
  - **Funciones**: `Call`, `Lambda`
  - **Datos**: `Tuple`, `Slice`, `Identifier`, `IntegerLiteral`, `FloatLiteral`
  - **Asignación**: `Assigment` (con flag monadic para notación `<-`)

- **`ConcreteSliceExpression`**: Slices con rango o lista de elementos

- **`BinaryOperator`**: Operadores binarios (Apply, Pipe, Add, Multiply, etc.)

#### **`normalized_ast.rs`**: Segunda Representación (tras Normalizador)
Forma intermedia más simplificada:

- **`NormalizedAST`**
  - `declarations: Vec<Declaration<Spanned>>`
  - Usa los tipos base (`Declaration` genérico) parametrizados con `Spanned`
  - El normalizador **desugaring** expresiones concretas complejas

#### **`typed_ast.rs`**: Tercera Representación (tras Type Inferrer)
Forma completamente tipada:

- **`Type` enum**: Tipos después de inferencia
  - `Builtin(String, Vec<Type>)`: Tipo builtin con parámetros (ej: `List[Int]`)
  - `Unknown(usize)`: Variable de tipo no resuelta (usado durante inferencia)
  - `Arrow(Box<Type>, Box<Type>)`: Tipo función
  - `Tuple(Vec<Type>)`: Tipo tupla
  - `Slice(Box<Type>)`: Tipo slice
  
- **`TypeMetadata`**: Metadatos para nodos tipados
  - `span: Span`: Ubicación original
  - `ttype: Type`: Tipo inferido
  
- **`Typed` impl de `ASTMetadata`**: 
  - `DeclarationMeta` = `TypeMetadata`
  - `ExpressionMeta` = `TypeMetadata`
  - `PatternMeta` = `Span`
  - `TypeMeta` = `TypeMetadata`
  
- **`TypedAST`**
  - `declarations: Vec<Declaration<Typed>>`
  - Completamente tipado, listo para backend

---

### 3. **qalc-parser** (Análisis Sintáctico)
**Propósito**: Transformar texto fuente → ConcreteAST

**Ubicación**: `crates/qalc-parser/src/`

**Dependencias principales**:
- `pest 2.8.6`: Parser PEG (Parsing Expression Grammar)
- `pest_derive 2.8.6`: Macros para definir gramática

**Módulos principales**:

#### **`lib.rs`**: Punto de Entrada
- **`parse_program(source: &Rc<Source>) -> CompilerResult<ConcreteAST>`**
  - Función pública principal
  - Tokeniza con `tokenize()` luego parsea declaraciones
  - Retorna árbol concreto o error

#### **`tokenizer.rs`**: Análisis Léxico con Pest
- **`Token<'a>`**: Wrapper sobre `Pair` de Pest
  - Mantiene referencia al `Source` original
  - Métodos: `span()`, `rule()`, `inner()`, `source()`
  
- **`TokensIter<'a>`**: Iterador de tokens
  - Implementa `Iterator`, `DoubleEndedIterator`, `ExactSizeIterator`
  - Métodos: `eat()` (pop siguiente), `eat_back()` (pop último)
  
- **`QalcGrammar`**: Parser derivado de PEG en `grammar.pest`
  
- **`tokenize(source) -> CompilerResult<TokensIter>`**
  - Parsea el programa usando la gramática PEG
  - Retorna iterador de tokens de declaraciones

#### **`grammar.pest`**: Especificación de Gramática (Pest)
Define la sintaxis del lenguaje Qalc:

**Literals**:
- Números enteros: decimales, hexadecimales (`0x...`), binarios (`0b...`)
- Sufijos de tipo: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`
- Números flotantes: decimal, exponencial, sufijos `f32`, `f64`

**Operadores**:
- **Aplicación**: `$` (apply), `|>` (pipe)
- **Lógicos**: `||` (or), `&&` (and)
- **Comparación**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Aritméticos**: `+`, `-`, `*`, `/`, `%`, `**` (power)
- **Bitwise**: `|`, `&`, `^`

**Expresiones**:
- Identifiers simples o operadores backtick (`` `+` ``)
- Patrones: variables, tuplas, wildcard
- Operaciones parciales: `(+ 5)`, `(5 *)` (bind operador con un lado)
- Lambdas: `(x y) -> x + y`
- Tuplas: `()`, `(a, b)`, `(a, b, c)`
- Slices: `[1, 2, 3]` o `[1..10]`

**Bloques**:
- `let x = 1, y = 2 in expr` (bindings)
- `if cond then a else b` (condicionales)
- `match expr { pat1 -> a, pat2 -> b }` (pattern matching)
- `perform { stmt1; stmt2; return value }` (bloque monádico)
- `linear { stmt1; stmt2 }` (contexto de tipo lineal)

#### **`declarations.rs`**: Parseo de Declaraciones
- **`parse_declaration(token: Token) -> CompilerResult<ConcreteDeclaration>`**
  - Discrimina entre `type` y function declarations
  - Llama a `parse_type_declaration()` o `parse_function_declaration()`
  
- Valida que nombre de tipo/función coincida con definición

#### **`expression/mod.rs`**: Parseo de Expresiones
Módulo con múltiples subarchivos:

- **`parse_expression()`**: Punto de entrada recursivo
  - Usa `PrattParser` para manejar precedencia y asociatividad
  
- **`atoms.rs`**: Expresiones atómicas (base)
  - Identificadores, literales, parentesizadas
  
- **`blocks.rs`**: Parseo de bloques
  - `let`, `if/else`, `match`, `perform`, `linear`
  
- **`numbers.rs`**: Parseo de literales numéricos
  
- **`utils.rs`**: Utilidades de parseo

#### **`pattern.rs`**: Parseo de Patrones
- **`parse_pattern(token: Token) -> Pattern<Spanned>`**
  - Reconoce variables, tuples, wildcards

#### **`type_expression.rs`**: Parseo de Tipos
- **`parse_type_expression(token: Token) -> TypeExpression<Spanned>`**
  - Identificadores simples o con parámetros
  - Tipos slice
  - Tuplas de tipos
  - Tipos función (Transform)

#### **`shared.rs`**: Funciones Compartidas
- Conversión de errores Pest → `Diagnostic`

---

### 4. **qalc-analysis** (Análisis Semántico)
**Propósito**: Transformar ConcreteAST → NormalizedAST → TypedAST con análisis semántico

**Ubicación**: `crates/qalc-analysis/src/`

**Módulos principales**:

#### **`normalizer/mod.rs`**: Normalizador (ConcreteAST → NormalizedAST)
- **`Normalizer`**: Struct que implementa `CompilerStep`
  - `Input = ConcreteAST`
  - `Output = NormalizedAST`
  
- **`transform(input, diagnostics) -> CompilerResult<NormalizedAST>`**
  - Itera declaraciones concretas
  - Llama a `transform_declaration()` para cada una
  
- **Submódulos del normalizador**:
  - `decl.rs`: Normalización de declaraciones
  - `expr.rs`: Desugaring y normalización de expresiones
  - `desugar.rs`: Conversión de características sintácticas
  - `adapters.rs`: Conversión entre tipos AST

**¿Qué hace la normalización?**
- **Desugaring**: Expande características sintácticas azúcar
- **Simplificación**: Reduce expresiones complejas a formas normalizadas
- **Validación**: Chequea consistencias semánticas iniciales
- Mantiene `Span` de ubicación para diagnósticos

#### **`hm_inferrer/mod.rs`**: Inferidor de Tipos Hindley-Milner (NormalizedAST → TypedAST)
- **`HMInferrer`**: Struct que implementa `CompilerStep`
  - `Input = NormalizedAST`
  - `Output = TypedAST`
  
- **Estado actual**: `transform()` retorna `todo!()` - **NO IMPLEMENTADO**

**¿Qué hará cuando esté implementado?**
- Inferencia de tipos usando algoritmo Hindley-Milner
- Resolución de variables de tipo
- Chequeo de tipos polimórficos
- Detección de ambigüedades
- Generación de metadatos de tipo para TypedAST

---

### 5. **qalc-cli** (Interfaz de Línea de Comandos)
**Propósito**: Ejecutable principal que orquesta el compilador

**Ubicación**: `crates/qalc-cli/src/main.rs`

**Flujo**:
1. Recibe nombre de archivo como argumento
2. Carga archivo fuente en `Source`
3. **Parser**: Llama `parse_program()` → ConcreteAST
4. Imprime ConcreteAST (debug output)
5. **Normalizer**: Crea `Normalizer::default()`, llama `transform()`
6. Imprime NormalizedAST (debug output)
7. Actualmente se detiene aquí (HM type inferrer no implementado)

**Ejemplo de uso**:
```bash
cargo run --bin qalc-cli program.qalc
```

---

## Sistema de Metadatos del AST

### Patrón de Diseño: AST Parametrizado

El compilador usa un patrón sofisticado donde la **misma estructura AST** puede contener diferentes metadatos según la fase:

```rust
pub trait ASTMetadata: Debug + Clone {
  type DeclarationMeta;
  type ExpressionMeta;
  type PatternMeta;
  type TypeMeta;
}

// Ejemplo 1: Fases tempranas (Spanned)
pub struct Spanned;
impl ASTMetadata for Spanned {
  type DeclarationMeta = Span;
  type ExpressionMeta = Span;
  type TypeMeta = Span;
  type PatternMeta = Span;
}

// Ejemplo 2: Fase tipada (Typed)
pub struct Typed;
impl ASTMetadata for Typed {
  type DeclarationMeta = TypeMetadata; // contiene Type + Span
  type ExpressionMeta = TypeMetadata;
  type TypeMeta = TypeMetadata;
  type PatternMeta = Span;
}
```

**Ventajas**:
- Una sola definición de árbol reutilizable
- Tipo-seguro: el compilador verifica que metadatos sean válidos
- Sin overhead de runtime

---

## Flujo de Compilación Detallado

### Fase 1: Parser (qalc-parser)
```
Archivo de texto
    ↓
Gramática PEG (grammar.pest)
    ↓
Tokenización léxica con Pest
    ↓
Parseo sintáctico recursivo
    ↓
ConcreteAST (con Span para cada nodo)
    ↓
Conserva toda estructura sintáctica, incluyendo azúcar
```

**Salida**: `ConcreteAST { declarations: Vec<ConcreteDeclaration> }`

### Fase 2: Normalización (qalc-analysis::normalizer)
```
ConcreteAST
    ↓
Análisis de declaraciones
    ↓
Desugaring de expresiones
    ↓
Simplificación de bloques
    ↓
Validación semántica inicial
    ↓
NormalizedAST (con Span, estructura más regular)
    ↓
Reduce complejidad sintáctica
```

**Salida**: `NormalizedAST { declarations: Vec<Declaration<Spanned>> }`

### Fase 3: Type Inference (qalc-analysis::hm_inferrer) - TODO
```
NormalizedAST
    ↓
Recolección de constraints de tipo
    ↓
Unificación de tipos (Hindley-Milner)
    ↓
Resolución de variables de tipo
    ↓
Generación de TypeMetadata
    ↓
TypedAST (con información de tipo completa)
    ↓
Código totalmente tipado y validado
```

**Salida esperada**: `TypedAST { declarations: Vec<Declaration<Typed>> }`

---

## Características del Lenguaje Qalc

### Tipos de Datos
- **Básicos**: Integer (con sufijos de tipo), Float
- **Compuestos**: Tuplas, Slices/Arrays
- **Funcionales**: Tipos función (Arrow)
- **Generics**: Tipos parametrizados (ej: `List[Int]`)
- **Lineales**: Flag para restricción de uso único

### Control de Flujo
- **Condicionales**: `if-then-else`
- **Pattern matching**: `match` con múltiples casos
- **Bindings**: `let-in` para asignaciones locales
- **Monádicas**: `perform` para operaciones secuenciales
- **Lineales**: `linear` para contextos de tipo lineal

### Expresiones
- **Lambdas**: `(x y) -> x + y` (funciones anónimas)
- **Aplicación parcial**: `(+ 5)` vincula operador con un operando
- **Pipe**: `a |> f` aplica función en estilo fluido
- **Apply**: `f $ a` aplicación explícita

### Operadores
- **Aritméticos**: `+`, `-`, `*`, `/`, `%`, `**`
- **Comparación**: `==`, `!=`, `<`, `>`, `<=`, `>=`
- **Lógicos**: `&&`, `||`
- **Bitwise**: `&`, `|`, `^`
- **Sobrecargables**: Usando backticks (`` `op` ``)

### Patrones
- **Variable**: Captura valor
- **Tuple**: Pattern matching estructural
- **Wildcard** (`_`): Ignora valor

---

## Tipos Principales Reutilizables

### `Declaration<M: ASTMetadata>`
Representa una declaración de alto nivel:
- Tipo de dato
- Función

Parametrizado para adaptarse a múltiples representaciones.

### `Expression<M: ASTMetadata>`
Base genérica para expresiones (usado en definiciones).

### `ConcreteExpression`
Expresión específica del AST concreto con muchas variantes sintácticas.

### `Pattern<M>`
Patrón usado en pattern matching y vinculación.

### `TypeExpression<M>`
Expresión de tipo (anotaciones y tipos en declaraciones).

### `Span`
Ubicación en el código fuente: `{ source, start, end }`
- Usado para diagnósticos y mensajes de error
- Se propaga a través del árbol

---

## Sistema de Diagnósticos

### Estructura
```rust
pub struct Diagnostic {
  kind: DiagnosticKind,      // Error, Warning, Note
  message: String,            // Mensaje principal
  label: String,              // Etiqueta contextual
  span: Span,                 // Ubicación en código
}

pub type CompilerResult<T> = Result<T, Diagnostic>;
```

### Características
- **Rendering bonito**: Integración con `ariadne`
- **Contextualizado**: Muestra línea de código con puntero
- **Color**: Según tipo de diagnóstico (rojo/amarillo/azul)
- **Propagación**: Las fases retornan `CompilerResult`

### Ejemplo de error:
```
Error in program.qalc:5:10
  ┌─ program.qalc:5:10
  │
5 │ let x = 1
  │          ^ Type mismatch: expected String, found Integer
```

---

## Dependencias Externas

### qalc-utils
- **ariadne 0.6.0**: Reportes de errores con colores y contexto
- **derive_more 2.1.1**: Derivación automática de Debug, Into, etc.

### qalc-parser
- **pest 2.8.6**: Parser PEG (sintaxis basada en gramática)
- **pest_derive 2.8.6**: Macros #[derive(Parser)]
- **lazy_static 1.5.0**: Static initializer para PRATT_PARSER

### qalc-ast, qalc-analysis, qalc-cli
- Solo dependencias internas (workspace crates)

---

## Puntos de Extensión Futuros

### No Implementados (TODO)
1. **HM Type Inferrer**: El algoritmo Hindley-Milner está esqueletizado
2. **Type checking completo**: Validación de tipos polimórficos
3. **Backend de código**: Generación de output (LLVM, JavaScript, etc.)
4. **Optimizaciones**: Dead code elimination, constant folding, etc.
5. **Información de scope**: Variables globales y locales (comentado en Typed)

### Características Futuras Posibles
- Módulos y espacios de nombres
- Traits/Typeclasses
- Generics más complejos
- Macros
- Import/Export de tipos y funciones

---

## Convenciones y Patrones

### Naming
- **Módulos public**: Contienen tipos y funciones reutilizables
- **Módulos private**: Lógica interna de cada crate
- **`mod.rs`**: Punto de entrada de un módulo

### Error Handling
- Todo usa `CompilerResult<T>` (Result con Diagnostic)
- No hay panic en lógica de compilación, todo retorna Err
- Los errores llevan ubicación exacta (Span)

### Iteración de Tokens
- `Token::eat()`: Pop siguiente (panic si no hay)
- `Token::eat_back()`: Pop último
- `Token::inner()`: Itera subnodos

### Span Tracking
- Todo nodo AST contiene Span
- Spans se unen (join) para expresiones compuestas
- Base para mensajes de error precisos

---

## Organización del Código

```
qalc/
├── Cargo.toml                    # Workspace root
├── README.md
├── rustfmt.toml
├── crates/
│   ├── qalc-utils/               # Utilidades base
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── source.rs         # Gestión de código fuente
│   │       ├── span.rs           # Ubicaciones en código
│   │       ├── diagnostic.rs     # Sistema de errores
│   │       └── compiler_step.rs  # Trait de fases
│   │
│   ├── qalc-ast/                 # Definiciones de AST
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ast_metadata.rs   # Metadatos parametrizados
│   │       ├── concrete_ast.rs   # AST del parser
│   │       ├── normalized_ast.rs # AST normalizado
│   │       ├── typed_ast.rs      # AST tipado
│   │       └── base_ast/         # Tipos base reutilizables
│   │
│   ├── qalc-parser/              # Parser sintáctico
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tokenizer.rs      # Análisis léxico (Pest)
│   │   │   ├── grammar.pest      # Especificación PEG
│   │   │   ├── declarations.rs
│   │   │   ├── pattern.rs
│   │   │   ├── type_expression.rs
│   │   │   ├── shared.rs
│   │   │   └── expression/
│   │   │       ├── mod.rs
│   │   │       ├── atoms.rs
│   │   │       ├── blocks.rs
│   │   │       ├── numbers.rs
│   │   │       └── utils.rs
│   │   └── tests/                # Test suite
│   │
│   ├── qalc-analysis/            # Análisis semántico
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── normalizer/       # Normalizador
│   │       │   ├── mod.rs
│   │       │   ├── decl.rs
│   │       │   ├── expr.rs
│   │       │   ├── desugar.rs
│   │       │   └── adapters.rs
│   │       └── hm_inferrer/      # Type inference (TODO)
│   │           └── mod.rs
│   │
│   └── qalc-cli/                 # Ejecutable principal
│       └── src/
│           └── main.rs
│
├── docs/                         # Documentación web (Astro)
└── target/                       # Build artifacts
```

---

## Cómo Construir y Ejecutar

### Build
```bash
cargo build          # Debug build
cargo build --release # Release build
```

### Ejecutar CLI
```bash
cargo run -p qalc-cli -- archivo.qalc
```

### Tests
```bash
cargo test           # Todos los tests
cargo test -p qalc-parser  # Tests solo del parser
```

---

## Resumen Ejecutivo

**Qalc** es un compilador educativo/experimental que implementa:

1. **Análisis léxico y sintáctico** con gramática PEG (Pest)
2. **Representación intermedia múltiple** usando AST parametrizado
3. **Normalización** y desugaring de características sintácticas
4. **Inferencia de tipos** (esqueletizado, requiere implementación)
5. **Sistema de diagnósticos robusto** con ubicaciones exactas

La arquitectura es **modular**, **extensible** y **type-safe**, permitiendo agregar nuevas fases (optimizaciones, backends, etc.) siguiendo el patrón `CompilerStep`.

Aunque está incompleto (especialmente HM inferrer), proporciona una base sólida para un compilador funcional con tipos fuertes.
