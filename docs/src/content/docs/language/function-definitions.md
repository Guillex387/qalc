---
title: Function definitions
description: Types, parameters, and function declarations.
---

Functions are declared with a type annotation followed by a definition.
The annotation and definition appear together, in that order.

## Type annotation

```qalc
sum :: i32 -> i32 -> i32
```

The arrow indicates input and output types. A function with multiple
arguments is a chain of single-argument functions.

## Definition

```qalc
sum :: i32 -> i32 -> i32
sum x y = x + y
```

The function name is repeated and parameters follow it.

## Zero-argument functions

```qalc
pi :: i32
pi = 314
```

## Operators as functions

You can refer to operators with backticks to treat them as functions.

```qalc
add :: i32 -> i32 -> i32
add = `+`
```

## Compound types

The parser supports tuples, slices, and functions.

```qalc
pairAdd :: (i32, i32) -> i32
pairAdd p = 0

listSum :: [i32] -> i32
listSum xs = 0
```

Note: adjust examples to the real types when the type system is complete.
