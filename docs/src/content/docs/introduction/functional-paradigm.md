---
title: Functional paradigm
description: Functional principles applied to qalc.
---

The functional approach helps you write predictable programs that are easy to compose.
In qalc, that means pure functions, immutability, and explicit data flow.

## Functions as values

A function can be stored in a variable, passed as an argument, or returned.
This enables modular and reusable logic.

```qalc
add :: i32 -> i32 -> i32
add x y = x + y

applyTwice :: (i32 -> i32) -> i32 -> i32
applyTwice f x = f (f x)
```

## Immutability

Values are not mutated. Instead of changing data, you create new values.
This reduces side effects and makes code easier to reason about.

```qalc
let x = 10
in x + 5
```

## Composition

Combine small functions to create more complex behavior.
The `|>` operator helps chain transformations.

```qalc
inc :: i32 -> i32
inc x = x + 1

double :: i32 -> i32
double x = x * 2

result = 10 |> inc |> double
```

## Control flow as an expression

Blocks like `if`, `match`, or `let` return a value, which allows
chaining decisions without leaving the expression style.

```qalc
sign :: i32 -> i32
sign x = if x < 0 then -1 else 1
```
