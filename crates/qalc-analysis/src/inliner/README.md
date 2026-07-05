# El inliner (aplanado por thunks)

Documento técnico de qué hace y qué está implementado en el **inliner** de Qalc
(`crates/qalc-analysis/src/inliner/`).

---

## 1. Propósito y lugar en el pipeline

El inliner es la fase que **aplana** el programa: partiendo de la función
`main`, inlina (beta-reduce) todas las llamadas a funciones **no built-in**
hasta dejar una **única expresión**. Es el paso previo a la generación de código
(objetivo a largo plazo: cuántico).

```
Source
  → [parser]          ConcreteAST
  → [normalizer]      NormalizedAST      (desazucarado: Transform currificados, sin match/perform)
  → [recursion_checker] NormalizedAST    (garantiza que NO hay recursión entre funciones)
  → [hm_inferrer]     TypedAST           (inferencia HM: cada nodo lleva su tipo)
  → [inliner]         FlatAST            ← esta fase
```

Que se ejecute **después del `RecursionChecker`** es lo que garantiza que
inlinar **termina**: sin ciclos entre funciones de usuario, la expansión es
finita.

Implementa el trait `CompilerStep`:

```rust
impl CompilerStep for Inliner {
  type Input = TypedAST;
  type Output = FlatAST;
}
```

---

## 2. La estructura de salida: `FlatAST`

Definida en `crates/qalc-ast/src/flat_ast.rs`.

```rust
pub struct FlatAST {
  pub params: Vec<FlatParam>,     // los n parámetros de entrada (de main)
  pub body: Expression<Flat>,     // la única expresión aplanada
  pub result: FlatResult,         // los m resultados
  pub global_scope: Rc<Scope>,    // scope global (se arrastra desde TypedAST)
}

pub struct FlatParam { pub name: String, pub ttype: Type }
pub struct FlatResult { pub arity: usize, pub qio: bool }
```

### Reutiliza la expresión base

El FlatAST **no define un enum de expresiones nuevo**: reutiliza
`base_ast::Expression<M>` con un marcador de metadata propio, `Flat`. Es decir,
la expresión aplanada conserva todas las variantes de `ExpressionKind`,
**incluida `Lambda`** (necesaria para los lambdas que sobreviven, ver §6).

```rust
pub struct Flat;
impl ASTMetadata for Flat {
  type DeclarationMeta = FlatMetadata;
  type ExpressionMeta = FlatMetadata;
  type PatternMeta = Span;
  type TypeMeta = Span;
}

pub struct FlatMetadata {
  pub span: Span,
  pub ttype: Type,      // tipo resuelto, arrastrado desde la fase Typed
  pub mark: FlatMark,   // marca (ver abajo)
}

pub enum FlatMark {
  Normal,
  ResidualLambda,       // lambda que sobrevive al inlining
}
```

Respecto a la metadata tipada (`TypeMetadata`), `FlatMetadata` **añade la marca
`FlatMark`** y **omite el `scope` por-nodo** (redundante en un programa ya
aplanado; el scope global sigue en `FlatAST::global_scope`).

---

## 3. Punto de entrada: `main`

El inliner introduce el concepto de **punto de entrada**, que antes no existía en
el compilador: busca la declaración de función llamada `"main"`. Si no existe,
devuelve un `Diagnostic` de error.

A partir de `main` calcula:

- **`params` (los n)**: pela los primeros `n = main.params.len()` `Type::Arrow`
  del tipo de `main` para obtener el tipo de cada parámetro. Los nombres vienen
  de `main.params` (que el normalizador ya ha renombrado a `p0, p1, …`, con el
  nombre original convertido en un `let` de destructuring dentro del cuerpo).
- **`body`**: `inline_expr(main.body, …)` con los parámetros de `main` marcados
  como locales.
- **`result` (los m)**: se calcula a partir del tipo de retorno de `main` (lo que
  queda tras pelar los `n` arrows):
  - `arity`: si el tipo relevante es `Type::Tuple(ts)` → `ts.len()`; si no → `1`.
  - `qio`: `true` si el tipo es `Type::Builtin { id: "qio", .. }`; como respaldo
    (mientras el sistema de tipos no modele `qio` con semántica propia) también
    se mira la anotación sintáctica de `main`. Si es `qio`, el tipo interno
    determina la aridad.

---

## 4. El recorrido: `inline_expr`

`inline_expr(expr, funcs, shadowed, lambdas)` recorre exhaustivamente la
expresión tipada y produce la expresión aplanada. Convierte la metadata
`TypeMetadata → FlatMetadata` (marca `Normal` por defecto).

Lleva dos estructuras de **entorno** que siguen el ámbito léxico (se clonan al
entrar en sub-ámbitos):

- **`shadowed: HashSet<String>`** — nombres ligados localmente (parámetros,
  lambdas, lets) que **ocultan** una función global homónima; no deben inlinarse
  como global.
- **`lambdas: HashMap<String, LocalLambda>`** — nombres locales que están ligados
  a una lambda (ver §6.2).

Y un mapa inmutable de las funciones de usuario disponibles para inlinar:

- **`funcs: HashMap<String, FnDef>`** — construido a partir de las declaraciones
  del `TypedAST` (`FnDef { params, body, ttype, annotation }`).

### 4.1 Clasificación de aplicaciones

En Qalc **toda** aplicación es un `Transform { callee, param }` currificado
(`f a b` = `Transform(Transform(f, a), b)`). No existe un nodo "call" ni una
marca "es builtin": la distinción se hace por **resolución de nombres**.

`classify_transform` mira la **cabeza de la espina** de aplicación y cuenta los
argumentos, y clasifica en:

| Clase | Cuándo | Acción |
|---|---|---|
| **Reducible** | La cabeza es una función de usuario, una lambda local o un lambda literal, **completamente aplicada** (aridad = nº de args) | Se beta-reduce (§5) |
| **Curry** | Lo mismo pero con aridad ≠ nº de args (aplicación parcial o sobre-aplicación) | `Diagnostic` de error (no soportado, ver §7) |
| **Other** | La cabeza es un builtin, un parámetro o un local no-lambda | Se conserva como `Transform`, recursando en `callee` y `param` |

Las **llamadas a builtins** (`add`, `monadMap`, `+`, …) caen en `Other`: no están
en `funcs`, así que se preservan tal cual. Ahí está la clave de "aplanar solo lo
que no es built-in".

Prioridad: un **local ligado a una lambda** se comprueba **antes** que una
función global homónima (el local la oculta).

### 4.2 Distinción builtin vs función de usuario

- **Función de usuario** = su nombre está en `funcs` y **no** está en `shadowed`.
- **Builtin / parámetro / local** = todo lo demás.

No hay ninguna bandera en el nodo; es exactamente el criterio "conjunto de
funciones globales + shadowing léxico" que ya usa el `RecursionChecker`.

---

## 5. La reducción: `reduce` (thunks por `let` no estricto)

Cuando una aplicación es **Reducible**, `peel_spine` desenrolla la espina en
`(cabeza, [args])` y `reduce` construye el resultado:

- Por cada `(param, arg)`:
  1. Se renombra `param` a un **nombre fresco** único (`format!("{param}_{}", gen_id())`)
     dentro del cuerpo, para **evitar la captura de variables** al aplanar
     (`rename_in_expr`, que respeta el shadowing léxico).
  2. El argumento se liga a ese nombre fresco mediante un **`LetBlock` no
     estricto** (`strict: false`) — el "thunk perezoso" a nivel de AST.
- El cuerpo inlinado va como `value` del `let`.

```
f a b   con   f x y = <body>
  ⇒   let x_N = a; y_M = b in <body[x→x_N, y→y_M] inlinado>
```

> **Sobre los "thunks":** no se implementa ninguna maquinaria de thunk en tiempo
> de ejecución. El `let strict:false` es solo la **representación** del thunk a
> nivel de AST (una ligadura perezosa, marcada); el forzado/sharing real
> correspondería a un evaluador o backend posterior que aún no existe. Frente a
> la sustitución directa, el `let` aporta ligar el argumento **una sola vez**
> (evita duplicar la expresión del argumento).

La misma maquinaria sirve para tres casos de cabeza reducible:
- función de usuario (cuerpo tomado de `funcs`),
- lambda local (cuerpo tomado de `lambdas`, ver §6.2),
- lambda literal en posición de cabeza (`(\x -> …) a`, beta-reducción directa).

Si tras reducir no queda ninguna ligadura (p. ej. todos los argumentos eran
lambdas propagadas), no se emite un `let` vacío: se devuelve el cuerpo inlinado
directamente.

---

## 6. Lambdas

Hay dos tratamientos, según se pueda o no eliminar la lambda.

### 6.1 Lambdas residuales (no eliminables)

Un lambda **en posición de valor que no se puede reducir** —típicamente el
callback de un **builtin de orden superior** como `monadMap` (presente en todo
`perform`/`qio`)— **se conserva** como `ExpressionKind::Lambda`, marcado
`FlatMark::ResidualLambda`. Dentro de su cuerpo se **sigue inlinando**
(reduciendo las llamadas que haya).

```
hof ((z) -> g z)      con hof builtin y g de usuario
  ⇒   hof ((z) -> <g inlinada>)      [la lambda queda, marcada residual]
```

Esto es lo que hace que los programas monádicos/`qio` sí se aplanen: la llamada
al builtin y su lambda se preservan, pero el interior se inlina.

### 6.2 Lambdas locales (orden superior indirecto)

Cuando una lambda se pasa a una **función de usuario** que la aplica, al inlinar
esa función la lambda queda ligada a un parámetro/local (`let post = <lambda>`) y
la llamada `post x` **no** es una lambda en posición de cabeza directa. Para
aplanarla igualmente, el inliner mantiene el mapa **`lambdas`**:

- Una ligadura `x = <lambda>` (o `x = <otro-local-lambda>`, propagación por
  copia) **se registra** en `lambdas` y **no se emite** como binding.
- Un argumento de función que es una lambda se **propaga** en `lambdas` en vez de
  ligarse como thunk.
- Cuando un nombre de `lambdas` aparece:
  - **como cabeza de aplicación** → se **beta-reduce** con la lambda registrada;
  - **como valor** (no aplicado) → se **reconstruye** la lambda ahí (queda
    residual, §6.1).
- El shadowing se respeta: reasignar el nombre a un no-lambda lo saca de
  `lambdas`; los parámetros de un lambda ocultan locales homónimos.

Ejemplo (el caso real que motivó esta parte):

```qalc
addWithPost a b post = add a b |> post;   -- aplica el parámetro-función
main a b = addWithPost a b ((a) -> (a, a));
```

`post (add a b)` con `post = (a) -> (a, a)` se reduce **por completo**:

```
let p0_23 = add a b in (let a = p0_23 in (a, a))
```

Sin lambda suelta, sin `residual`: la lambda desaparece.

---

## 7. Qué está soportado y qué no

**Soportado:**
- Inlinado (beta-reducción) de llamadas a funciones de usuario completamente
  aplicadas, con thunks (`let` no estricto) y renombrado anti-captura.
- Conservación de llamadas a builtins.
- Funciones nullary usadas como valor (se inlina su cuerpo).
- Lambdas residuales (argumento de builtin) conservadas y marcadas.
- Lambdas locales de orden superior: propagación y reducción, incluyendo
  propagación por copia y reconstrucción en posición de valor.
- Cálculo de `params` (n) y `result` (m, con detección de `qio`).

**No soportado / limitaciones conocidas:**
- **Currificación**: aplicación parcial o sobre-aplicación de funciones de
  usuario, y referencia a una función de usuario con parámetros **sin aplicar**
  (uso point-free / de orden superior de un global) → `Diagnostic` de error.
- **Captura por shadowing de libres**: al inlinar el cuerpo de una lambda local
  en su sitio de uso, si la lambda captura variables libres y esos nombres se
  re-ligan entre su definición y su uso, la reducción podría capturar mal. En
  callbacks típicos (sin libres o sin re-shadowing) no ocurre.
- **Duplicación**: una lambda local aplicada N veces duplica su cuerpo N veces
  (correcto semánticamente, solo afecta al tamaño).
- **Lambda recursiva** ligada por `let` podría no terminar (aunque el
  `RecursionChecker` normalmente ya la rechaza).
- **`qio`** aún no es un tipo con semántica propia en el sistema de tipos; la
  detección de `result.qio` es best-effort.
- El scope por-nodo **no** se conserva (solo el global en la raíz).

---

## 8. Ficheros

| Fichero | Contenido |
|---|---|
| `crates/qalc-ast/src/flat_ast.rs` | Tipos del FlatAST: `Flat`, `FlatMetadata`, `FlatMark`, `FlatAST`, `FlatParam`, `FlatResult` |
| `crates/qalc-analysis/src/inliner/mod.rs` | `Inliner` (`CompilerStep`), localización de `main`, cálculo de `params`/`result`, `FnDef` |
| `crates/qalc-analysis/src/inliner/expr.rs` | El recorrido: `inline_expr`, `classify_transform`, `peel_spine`, `reduce`, `rename_in_expr`, mapa `lambdas` (`LocalLambda`) |
| `crates/qalc-analysis/tests/inliner.rs` | Tests de integración (front-end completo → FlatAST) |
| `crates/qalc-cli/src/main.rs` | Ejecuta el paso e imprime el FlatAST como JSON reducido (`flat_ast_to_json`) |

---

## 9. Depuración

El CLI imprime el FlatAST como **JSON reducido** (sin scope ni spans), con el
tipo en cada nodo y `"residual": true` en los lambdas residuales:

```bash
cargo run -p qalc-cli -- ruta/al/fichero.qalc
```

Claves del JSON: `ident`, `int`/`float`, `app{fn,arg}`, `let{strict,bindings,in}`,
`if{test,then,else}`, `tuple`, `slice`/`range`, `lambda{params,body}` (+ `residual`),
más `params`, `result` y `type` en cada nodo.
