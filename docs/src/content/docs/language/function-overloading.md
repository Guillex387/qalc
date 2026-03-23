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
sum :: Int -> Int -> Int
sum x y = x + y

sum :: Qubit -> Qubit -> QOp
sum a b = applyQuantumAdd a b
```

Internal names:

```qalc
sum_Int
sum_Qubit
```

When the code says `sum x`, the compiler checks the type of `x`:

1. If `x` is `Int`, it substitutes `sum` with `sum_Int`.
2. Currying continues normally because the function type is now unambiguous.

## Ambiguity errors

If two overloads match the same argument types, the compiler reports an
ambiguity error and shows the conflicting signatures.
