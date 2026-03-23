---
title: Currying
description: Partial application and function composition.
---

In qalc, functions are curried by default. A function with multiple
parameters is interpreted as a chain of single-argument functions.

## Partial application

```qalc
sum :: i32 -> i32 -> i32
sum x y = x + y

add10 :: i32 -> i32
add10 = sum 10
```

## Pipe and apply

The `|>` operator applies a function to the value on the left. The `$` operator
controls associativity and reduces parentheses.

```qalc
inc :: i32 -> i32
inc x = x + 1

double :: i32 -> i32
double x = x * 2

result = 10 |> inc |> double
result2 = double $ inc 10
```

## Partial operators

You can build partial functions from operators.

```qalc
addLeft :: i32 -> i32
addLeft = (+ 10)

addRight :: i32 -> i32
addRight = (10 +)
```

## Lambdas

```qalc
mul :: i32 -> i32 -> i32
mul x y = x * y

square :: i32 -> i32
square = (x) => mul x x
```
