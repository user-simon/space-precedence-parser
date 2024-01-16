# Space Precedence Parser

This is a proof-of-concept of a parser that uses spacing to dictate operator precedence, wherein spaces can be seen as implicit parentheses. This has no practical applications. 

It's implemented as a modified [operator-precedence parser](https://en.wikipedia.org/wiki/Operator-precedence_parser), but where precedence is determined not only by the algebraic precedence of the operator (à la PEMDAS), but also by the amount of whitespace between the operator and the operand. 


## Examples

```
parse("1 * 2+3")
→ 1 * (2 + 3)

parse("sqrt  1 + 2")
→ sqrt (1 + 2)

parse("1-  2   *   3/4")
→ (1 - 2) * (3 / 4)

parse("2 + 4 * 6 - 8")
→ 2 + (4 * 6) - 8
```
