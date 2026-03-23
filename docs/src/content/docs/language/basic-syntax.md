---
title: Basic syntax
description: Literals, expressions, and essential blocks.
---

This page summarizes qalc base syntax, based on the current parser.

## Comments

```qalc
// Line comment
# Line comment
/* Block
  comment */
```

## Identifiers

- Start with an ASCII letter or `_`.
- Can then contain letters, numbers, or `_`.
- Reserved keywords cannot be used as identifiers.

## Numeric literals

qalc supports integers with type suffixes.

```qalc
42
0b1010
0xFF

10u8
20u16
30u32
40u64

-5i8
-6i16
-7i32
-8i64
```

## Operators

- Application: `$`
- Pipe: `|>`
- Logical: `||`, `&&`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
- Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
- Bitwise: `|`, `&`, `^`

## Expressions and application

Application is positional and space-separated.

```qalc
sum :: i32 -> i32 -> i32
sum x y = x + y

sum 3 4
```

You can also use `|>` to chain.

```qalc
inc :: i32 -> i32
inc x = x + 1

10 |> inc
```

## Blocks

All blocks are expressions and return a value.

```qalc
let x = 10
in x + 5
```

```qalc
if x > 0 then x else -x
```

```qalc
match x {
  0 -> 0,
  1 -> 1,
  default -> x
}
```

```qalc
perform {
  x = 1,
  y = 2,
  x + y
}
```

```qalc
linear {
  x = 2,
  x * 3
}
```
