# Y# Easy Syntax Reference

The Easy syntax (`.yse`) is a transpiled shorthand for standard Y# (`.ys`). It uses indentation-based blocks, automatic string quoting, and implicit function calls for concise, Python-like code.

---

## Indentation-Based Blocks

No curly braces `{}`. Indentation defines block boundaries.

```yse
// Standard .ys:
Function Main() -> Void
{
    if true
    {
        PrintLine("yes")
    }
}

// Easy .yse:
fn Main() -> Void
    if true
        PrintLine("yes")
```

---

## Auto-String

Bare text in expression position becomes a string literal. `$var` interpolates a variable.

```yse
PrintLine Hello World       // → PrintLine("Hello World")
PrintLine Value is: $x       // → PrintLine("Value is: " + x)
PrintLine $name is $age old  // → PrintLine(name + " is " + age + " old")
```

---

## Single Quotes → Double Quotes

Single-quoted strings become double-quoted.

```yse
PrintLine 'hello'    // → PrintLine("hello")
```

---

## Implicit Function Calls

A name followed by arguments without parentheses becomes a function call.

```yse
greet World           // → greet("World")
add 3 4               // → add(3, 4)
compare a b           // → compare(a, b)
```

### Chaining

```yse
println add 3 4       // → println(add(3, 4))
```

---

## Block Starters

These keywords start an indented block:

| Keyword     | Transpiles To                         |
|-------------|---------------------------------------|
| `fn`        | `Function`                            |
| `if`        | `if`                                  |
| `elif`      | `else if`                             |
| `else`      | `else`                                |
| `while`     | `while`                               |
| `loop`      | `Loop`                                |
| `for`       | `for`                                 |

```yse
fn Main() -> Void
    if x > 0
        PrintLine positive
    elif x < 0
        PrintLine negative
    else
        PrintLine zero
```

---

## No Semicolons

Semicolons are never needed. Line breaks end statements.

---

## Transpiler Debug

To see what `.yse` transpiles to:

```bash
ys easy-debug file.yse
```

This outputs the equivalent `.ys` code to stdout without compiling.

### Full Example

```yse
// factorial.yse
fn Factorial(n: Int) -> Int
    if n <= 1
        return 1
    n * Factorial(n - 1)

fn Main() -> Void
    PrintLine Factorial 5
```

Transpiles to:

```ys
Function Factorial(n: Int) -> Int
{
    if n <= 1
    {
        return 1
    }
    return n * Factorial(n - 1)
}

Function Main() -> Void
{
    PrintLine(Factorial(5))
}
```
