# Language Reference

## Program Structure

A Y# program consists of functions, types, and ECS declarations. Execution starts at `Main`.

```ys
// Entry point
Function Main() -> Void
{
    PrintLine("Hello")
}
```

---

## Types

| Type     | Description              | Example          |
|----------|--------------------------|------------------|
| `Int`    | Signed integer (64-bit)  | `42`, `-7`       |
| `Float`  | Double-precision float   | `3.14`, `-0.5`   |
| `Bool`   | Boolean                  | `true`, `false`  |
| `String` | UTF-8 string             | `"hello"`        |
| `Void`   | No return value          | —                |
| `_`      | Infer type from context  | —                |

### Type Inference

```ys
var x := 42          // Int
var y := 3.14        // Float
var z := "hello"     // String
var w: _ = true      // Bool (inferred from value)
```

---

## Variables

| Keyword  | Mutability         | Scope          |
|----------|--------------------|----------------|
| `var`    | Mutable            | Block         |
| `let`    | Immutable          | Block         |
| `const`  | Compile-time const | Global/Block  |

```ys
var count: Int = 0
let name: String = "Y#"
const MAX: Int = 100

count = count + 1   // OK
name = "other"       // Error: let is immutable
```

---

## Statements

### if / else

```ys
if condition { ... }
else if condition { ... }
else { ... }
```

### while

```ys
while condition { ... }
```

### Loop (infinite)

```ys
Loop { ... break }
```

### For

```ys
for var in start..end { ... }
for var in start..end..step { ... }
```

### return, break, continue

```ys
Function Square(x: Int) -> Int { return x * x }
// break and continue work inside while, Loop, for
```

---

## Functions

```ys
Function Name(param1: Type1, param2: Type2) -> ReturnType
{
    // body
    return value
}
```

### Examples

```ys
Function Add(a: Int, b: Int) -> Int
{
    return a + b
}

Function Log(msg: String) -> Void
{
    PrintLine("[LOG] " + msg)
}

// Default return is the last expression
Function Double(x: Int) -> Int
{
    x * 2
}
```

---

## Expressions

| Category   | Examples                          |
|------------|-----------------------------------|
| Binary     | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `\|\|` |
| Unary      | `-x`, `!condition`               |
| Call       | `FunctionName(args...)`           |
| Method     | `obj.Method(args...)`             |
| Field      | `obj.field`                       |
| Index      | `array[index]`                    |
| Block      | `{ stmt; expr }` → evaluates to last expr |

---

## Comments

```ys
// Line comment

/*
 * Block comment
 */
```

---

## ECS (Entity-Component-System)

```ys
Entity Player

Component Position
{
    var x: Float
    var y: Float
}

System Movement
{
    for each entity with Position
    {
        entity.Position.x += 1.0
    }
}
```

---

## Actor Model

```ys
Actor Counter
{
    var count: Int = 0

    Function Increment(amount: Int) -> Void
    {
        count = count + amount
    }

    Function GetCount() -> Int
    {
        return count
    }
}

// Usage
var c = Counter()
c.Increment(5)
var n = c.GetCount()  // 5
```

---

## GPU Compute (Kernel)

```ys
Kernel VecAdd(a: Float[], b: Float[], c: Float[], n: Int)
{
    var i = ThreadId.X
    if i < n
        c[i] = a[i] + b[i]
}

Function Main() -> Void
{
    var a: Float[1024]
    var b: Float[1024]
    var c: Float[1024]
    // ... initialize a, b ...
    VecAdd(a, b, c, 1024)  // dispatched on GPU
}
```
