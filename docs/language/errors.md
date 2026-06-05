# Error Messages

All Y# compiler errors, what they mean, and how to fix them.

---

## Parse Errors

### Expected keyword `Function`

**Message:** `expected keyword 'Function' at line X, column Y`

**Cause:** A top-level declaration doesn't start with a known keyword.

```ys
Main() -> Void { }  // Wrong
```

**Fix:** Add the `Function` keyword:

```ys
Function Main() -> Void { }
```

---

### Expected expression

**Message:** `expected expression at line X, column Y`

**Cause:** A statement position lacks a valid expression (e.g., dangling operator).

```ys
var x: Int = +   // Incomplete
```

**Fix:** Provide a complete expression:

```ys
var x: Int = 42
```

---

### Unexpected token

**Message:** `unexpected token '...' at line X, column Y`

**Cause:** A token appears where it doesn't belong (e.g., `}` in the middle of an expression).

```ys
var x = 42 } + 1
```

**Fix:** Remove the stray token.

---

### Expected type annotation

**Message:** `expected type annotation at line X, column Y`

**Cause:** A parameter or variable declaration is missing its type and cannot be inferred.

```ys
Function Add(a, b: Int) -> Int { return a + b }
```

**Fix:** Annotate all parameters:

```ys
Function Add(a: Int, b: Int) -> Int { return a + b }
```

---

### Expected `->` in function declaration

**Message:** `expected '->' in function declaration`

**Cause:** A function declaration is missing the return type arrow.

```ys
Function Add(a: Int, b: Int) Int { }
```

**Fix:** Add `->`:

```ys
Function Add(a: Int, b: Int) -> Int { }
```

---

### Unterminated string literal

**Message:** `unterminated string literal`

**Cause:** A string is missing its closing quote.

```ys
var s: String = "hello
```

**Fix:** Close the string:

```ys
var s: String = "hello"
```

---

### Expected `{` after `if` / `while` / `for` / `Loop`

**Message:** `expected '{' after 'if'`

**Cause:** A block-starting keyword is not followed by `{`.

```ys
if true PrintLine("yes")
```

**Fix:** Add braces:

```ys
if true { PrintLine("yes") }
```

---

### Expected `(` after function name

**Message:** `expected '(' after function name`

**Cause:** A function call is missing opening parenthesis.

```ys
PrintLine "hello"
```

**Fix:** Add parentheses:

```ys
PrintLine("hello")
```

---

### Invalid indentation (Easy mode)

**Message:** `inconsistent indentation at line X`

**Cause:** Tabs and spaces mixed, or indentation level doesn't match expected block depth.

```yse
fn Main() -> Void
    PrintLine("ok")
   PrintLine("bad")   # 3 spaces instead of 4
```

**Fix:** Use consistent indentation (spaces recommended).

---

## Type Errors

### Undefined variable

**Message:** `undefined variable 'x'`

**Cause:** Using a variable that has not been declared in the current scope.

```ys
PrintLine(x)  // x not declared
```

**Fix:** Declare the variable before use:

```ys
var x: Int = 42
PrintLine(x)
```

---

### Undefined function

**Message:** `undefined function 'Foo'`

**Cause:** Calling a function that has not been declared.

```ys
Foo()  // not defined
```

**Fix:** Define the function or check spelling:

```ys
Function Foo() -> Void { PrintLine("foo") }
```

---

### Type mismatch

**Message:** `type mismatch: expected Int, got String`

**Cause:** Passing a value of the wrong type where a specific type is expected.

```ys
Function Square(x: Int) -> Int { return x * x }
Square("hello")
```

**Fix:** Pass the correct type:

```ys
Square(42)
```

---

### Numeric operation on String

**Message:** `numeric operation '+' on String`

**Cause:** Using `+`, `-`, `*`, `/` on a string value (concatenation uses `+` only between strings; mixing types is an error).

```ys
var s: String = "hello" * 2
```

**Fix:** Use string-specific operations:

```ys
var s: String = StringRepeat("hello", 2)
```

---

### Cannot assign to immutable variable

**Message:** `cannot assign to immutable variable 'name'`

**Cause:** Trying to reassign a `let` variable.

```ys
let name: String = "Y#"
name = "Other"
```

**Fix:** Use `var` for mutable variables:

```ys
var name: String = "Y#"
name = "Other"
```

---

### Return type mismatch

**Message:** `expected return type Int, got String`

**Cause:** Function body returns a value that doesn't match the declared return type.

```ys
Function GetAge() -> Int { return "old" }
```

**Fix:** Return the correct type:

```ys
Function GetAge() -> Int { return 42 }
```

---

### Void function returning a value

**Message:** `void function should not return a value`

**Cause:** A `-> Void` function has `return expr`.

```ys
Function Log() -> Void { return 42 }
```

**Fix:** Use bare `return` or omit:

```ys
Function Log() -> Void { return }
```

---

### Index on non-array type

**Message:** `cannot index type Float`

**Cause:** Using `[]` on a type that doesn't support indexing.

```ys
var x: Float = 3.14
PrintLine(x[0])
```

**Fix:** Use index only on arrays or supported types.

---

### Cannot infer type

**Message:** `cannot infer type for variable 'x'`

**Cause:** Using `:=` or `_` where the type cannot be deduced from context.

```ys
var x := _
```

**Fix:** Provide an explicit value or type annotation:

```ys
var x: Int = 0
```

---

### Recursive type

**Message:** `recursive type 'X'`

**Cause:** A type (e.g., an Entity or struct) references itself in a way that cannot be resolved.

---

## Build Errors

### File not found

**Message:** `file not found: 'source.ys'`

**Cause:** The input file path doesn't exist.

**Fix:** Verify the file path:

```bash
ls source.ys
ys build source.ys
```

---

### GCC not found

**Message:** `GCC not found in PATH`

**Cause:** The C compiler (gcc) is not installed or not on PATH.

**Fix:** Install GCC (MinGW on Windows, build-essential on Linux, Xcode CLT on macOS).

```bash
# Linux
sudo apt install build-essential

# macOS
xcode-select --install

# Windows (MinGW)
# Add C:\MinGW\bin to PATH
```

---

### GCC compilation failed

**Message:** `GCC compilation failed with exit code X`

**Cause:** The generated C code failed to compile. Usually a compiler bug or missing linker symbols.

**Fix:** Check the generated C file in the build output. Ensure all linked libraries are available.

---

### Multiple definitions of `Main`

**Message:** `multiple definitions of 'Main'`

**Cause:** More than one `Function Main()` is defined.

**Fix:** Keep exactly one `Main` entry point.

---

### Invalid target

**Message:** `invalid target 'xyz'. Expected: native, game, gpu, wasm, llvm`

**Cause:** The `--target` flag received an unrecognized value.

**Fix:** Use a valid target:

```bash
ys build file.ys --target native
```

---

### Missing kernel attribute

**Message:** `function 'Foo' uses GPU intrinsics but is not declared as Kernel`

**Cause:** Using `ThreadId.X` or other GPU intrinsics outside a `Kernel` block.

**Fix:** Declare the function as `Kernel`:

```ys
Kernel Foo() { var i = ThreadId.X }
```

---

### Easy syntax in `.ys` file

**Message:** `file must have .yse extension to use easy syntax`

**Cause:** A `.ys` file contains indentation-based blocks or implicit calls.

**Fix:** Rename the file to `.yse` or use standard Y# syntax.

---

## Runtime Errors

### Division by zero

**Message:** `runtime error: integer division by zero`

**Cause:** Integer division by zero.

**Fix:** Check divisor before dividing:

```ys
if divisor != 0 { result = a / divisor }
```

### Index out of bounds

**Message:** `runtime error: index out of bounds: len=X, index=Y`

**Cause:** Accessing an array element beyond its length.

**Fix:** Bounds-check before access.

### Null reference

**Message:** `runtime error: null reference`

**Cause:** Accessing a field or method on a null object/pointer.

**Fix:** Ensure the value is initialized before use.
