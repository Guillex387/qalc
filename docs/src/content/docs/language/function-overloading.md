---
title: Function overloading
description: Define multiple variants for the same function.
---

qalc uses simple type-based overloading where operators and identifiers can map
to multiple function definitions. This avoids type-class style machinery and
fits a language with explicit types.

## Key idea

Overload resolution is name-based and driven by types, not by type inference.
The compiler resolves a name to the correct function using the type of the
first argument (or all arguments when needed).

## Symbol table model

Instead of a single function per name, the symbol table stores a list of
functions for each identifier or operator.

## Resolution rules

1. Collect all overloads by name.
2. Filter by the type of the first argument (or all arguments when required).
3. If exactly one candidate remains, select it.
4. If none match, report a clear type error.
5. If multiple candidates match, report an ambiguity error.

## Currying and name mangling

To keep currying predictable, the compiler internally generates a stable name
per overload (a static vtable or name mangling).

User code:

```qalc
sum :: i32 -> i32 -> i32
sum x y = x + y

sum :: (Qu8, Qu8) -> (Qu8, Qu8, Qu8)
sum (a, b) = someQaddCircuit (a, b)
```

Internal names:

```qalc
sum_i32
sum_Qu8
```

When the code says `sum x`, the compiler checks the type of `x`:

1. If `x` is `i32`, it substitutes `sum` with `sum_i32`.
2. If `x` is `Qu8`, it substitutes `sum` with `sum_Qu8`.
3. Currying continues normally because the function type is now unambiguous.

## Ambiguity errors

If two overloads match the same argument types, the compiler reports an
ambiguity error and shows the conflicting signatures.

## Overloading operators

Overloaded operators follow the same rules, using their operator names as the
symbol key.

```qalc
`+` :: i32 -> i32 -> i32
`+` x y = addI32 x y

`+` :: (Qu8, Qu8) -> (Qu8, Qu8, Qu8)
`+` (a, b) = someQaddCircuit (a, b)
```

## Notes on quantum boolean results

When an overload returns a quantum boolean (for example, `Qbool`), it should be
treated as a quantum value that is not measured unless explicitly requested.
