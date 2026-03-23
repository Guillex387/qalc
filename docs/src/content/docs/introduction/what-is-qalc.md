---
title: What is qalc
description: High-level overview of the language and its functional focus.
---

qalc is a functional language designed for high-level quantum calculations.
Its syntax favors composition and treats functions as first-class values.

## Key ideas

- Everything is an expression: each block returns a value.
- Functions apply by position, with a direct syntax.
- Types are declared explicitly with annotations.
- Composition is a central pattern for reusable logic.

## Quick look

```qalc
sum :: i32 -> i32 -> i32
sum x y = x + y

result = sum 10 20
```

```qalc
inc :: i32 -> i32
inc x = x + 1

result = 10 |> inc
```

## How to read this guide

1. Start with the functional paradigm introduction.
2. Continue with basic syntax and language blocks.
3. Review function definitions and currying.
4. Finish with overloading rules.
