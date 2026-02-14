# Chapter 11 · Math Expressions

[← Aliases](10-aliases.md) · [Next: Writing Scripts →](12-scripting.md)

---

Nes includes a built-in math expression evaluator, accessed through the `calc` command.

## Usage

```nes
calc <expression>
```

```nes
calc 2+2                       # 4
calc 10-3                      # 7
calc 6*7                       # 42
calc 100/4                     # 25
```

---

## Operators

| Operator | Name           | Example     | Result |
| -------- | -------------- | ----------- | ------ |
| `+`      | Addition       | `calc 2+3`  | `5`    |
| `-`      | Subtraction    | `calc 10-4` | `6`    |
| `*`      | Multiplication | `calc 6*7`  | `42`   |
| `/`      | Division       | `calc 15/4` | `3.75` |
| `%`      | Modulo         | `calc 17%5` | `2`    |
| `^`      | Power          | `calc 2^10` | `1024` |

---

## Precedence

From highest to lowest:

| Priority | Operators       | Associativity |
| -------- | --------------- | ------------- |
| 1        | `(` `)`         | —             |
| 2        | `+` `-` (unary) | Right         |
| 3        | `^`             | Right         |
| 4        | `*` `/` `%`     | Left          |
| 5        | `+` `-`         | Left          |

```nes
calc 2+3*4                     # 14   (not 20, multiplication first)
calc (2+3)*4                   # 20   (parentheses override)
calc 2^3^2                     # 512  (right-associative: 2^(3^2) = 2^9)
```

---

## Parentheses

Group expressions to control evaluation order:

```nes
calc (10+5)*2                  # 30
calc 100/(4+1)                 # 20
calc ((2+3)*(4+5))             # 45
calc (2^(1+2))                 # 8
```

Nested parentheses work to any depth.

---

## Unary Operators

Negative and positive prefixes:

```nes
calc -5+3                      # -2
calc -(3+4)                    # -7
calc +5                        # 5
calc -(-3)                     # 3
```

---

## Output Format

| Condition             | Format  | Example                            |
| --------------------- | ------- | ---------------------------------- |
| Whole number, < 10^15 | Integer | `calc 2^10` → `1024`               |
| Fractional            | Float   | `calc 10/3` → `3.3333333333333335` |

```nes
calc 10/2                      # 5       (integer)
calc 10/3                      # 3.3333333333333335  (float)
calc 2^10                      # 1024    (integer)
calc 1/7                       # 0.14285714285714285 (float)
```

---

## Spaces

Spaces are stripped before evaluation. All equivalent:

```nes
calc 2+3
calc 2 + 3
calc  2  +  3
```

---

## Errors

```nes
calc 10/0                      # calc: div/0
calc 10%0                      # calc: mod/0
calc 2+                        # calc: unexpected end
calc abc                       # calc: unexpected char
calc 2+(3                      # calc: missing )
```

---

## Practical Examples

```nes
# Unit conversions
calc 1024*1024                 # 1048576 bytes in a MB
calc 1024*1024*1024            # 1073741824 bytes in a GB
calc 2^32                      # 4294967296 (32-bit address space)

# Time
calc 365*24*60*60              # 31536000 seconds in a year
calc 24*60                     # 1440 minutes in a day

# Temperature
calc 9/5*100+32                # 212 (100°C → °F)
calc (72-32)*5/9               # 22.222... (72°F → °C)

# Percentages
calc (100-15)/100*200          # 170 (15% discount on $200)
calc 85/200*100                # 42.5 (85 out of 200 as %)
```

---

[← Aliases](10-aliases.md) · [Next: Writing Scripts →](12-scripting.md)
