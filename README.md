# Y# (YSharp) v8.0.5 — Oyster Shell

> *Y# (pronounced "why-sharp" or "oyster") is a modern systems programming language for games, AI, and high-performance computing.*

[![GitHub release](https://img.shields.io/github/v/release/ouzlifaneyassine1-dot/YSharp-YSharp)](https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases/tag/v8.0.5)

---

## Description

Y# is a compiled, statically-typed systems language with:
- **Built-in ECS architecture** (Entity/Component/System) for game development
- **GPU compute kernel compilation** (SPIR-V 1.6 backend)
- **Automatic differentiation** for ML/AI
- **Actor model** for concurrent systems
- **Tensor operations** and neural network primitives
- **Multiple codegen backends** (Native C, Game C++, GPU SPIR-V, WASM, LLVM)
- **Y# Easy** — an indentation-based transpiler dialect with auto-string print

### V8 Feature Set

| Category | Status |
|----------|--------|
| Polymorphic Print/PrintLine | ✓ |
| Variables (var/let/const) + type inference | ✓ |
| Conditionals (if/else) | ✓ |
| Numeric Loop (Loop i from x to y) | ✓ |
| While loops | ✓ |
| Functions with params + return types | ✓ |
| Function parameter passing (v8.0.5 fix) | ✓ |
| Binary/unary expressions | ✓ |
| Comments (//, /* */) | ✓ |
| Full compiler pipeline | ✓ |
| Y# Easy transpiler (.yse) | ✓ |
| Windows NSIS installer (dir + PATH + file association) | ✓ |
| Double-click .ys/.yse to build & run (Python-like) | ✓ |
| Unix shebang (#!) support | ✓ |
| ECS (Entity/Component/System) | Parser + HIR |
| Actor model | Parser + HIR |
| GPU compute (SPIR-V backend) | ✓ |
| MIR optimizer (const fold, LICM, vectorize, reorder) | ✓ |
| npm distribution package | ✓ |
| Package manager (oys pack) | ✓ |
| oys test runner | ✓ |

---

## Easy Syntax (.yse)

Y# Easy is a Python-style indentation-based dialect that transpiles to standard Y#. No braces `{}`, no semicolons `;`, no double-quotes needed for strings in print statements. Source files use the `.yse` extension.

### Quick Comparison

| Easy (.yse) | Standard (.ys) |
|---|---|
| `fn add a b -> Int` | `Function add(a: Int, b: Int) -> Int { ... }` |
| `if x > 5` | `if (x > 5) {` |
| `print Hello World` | `Print("Hello World");` |
| `println $name` | `PrintLine(name);` |
| `loop i from 1 to 10` | `Loop(i from 1 to 10) {` |
| `greet World` | `greet("World");` |

### Block-Introducing Keywords

These automatically get a `{` appended and their body is indented:

```
fn name params       → Function name(params) {
if cond              → if (cond) {
while cond           → while (cond) {
loop var from x to y → Loop(var from x to y) {
for item in coll     → for (item in coll) {
else if cond         → else if (cond) {
entity Name          → Entity Name {
system Name(Comp)    → System Name(Comp) {
actor Name           → Actor Name {
on Event             → On(Event) {
view                 → View {
state <Type> name    → State<Type> name {
```

### Rules

1. **Indentation**: 2 or 4 spaces — consistent per file. Dedent closes blocks with `}`.
2. **No semicolons**: Newlines end statements.
3. **Single quotes `'...'`** → converted to `"..."` (double quotes).
4. **`$var`** in print → variable interpolation (the `$` is stripped).
5. **`return value`** → `Return value;`.
6. **Implicit calls**: If a line looks like `name arg1 arg2` (not an assignment, not a keyword), it becomes `name("arg1", "arg2")`. Numbers, `$var`, and `true`/`false` are NOT quoted — they pass through literally.
7. **Blank lines** preserved. **`//` comment lines** preserved.
8. **Auto-wrap**: If no `Program` wrapper exists, the whole file is wrapped in `Program Main { ... }`.

### Examples

```yse
// hello.yse — no Program wrapper needed
fn main
    println Hello World

// Transpiles to:
// Program Main {
//     Function main() {
//         PrintLine("Hello World");
//     }
// }
```

```yse
fn greet name times
    loop i from 1 to times
        print Hello $name

// Transpiles to:
// Function greet(name, times) {
//     Loop(i from 1 to times) {
//         Print("Hello " + name);
//     }
// }
```

```yse
fn add a b -> Int
    return a + b

fn main
    var result = add 5 3
    println The answer is $result
```

```yse
// Auto-string print:
print Hello World      → Print("Hello World");
print $name            → Print(name);
println Hello $name    → PrintLine("Hello " + name);
println 'Explicit' var → PrintLine("Explicit" + var);
print                  → Print("");
```

### Debugging Easy

```bash
oys easy-debug file.yse    # Shows transpiled Y# output
```

---

## Standard Y# Syntax (.ys)

### Program Structure

Every `.ys` file starts with a `Program` block:

```ys
Program Main {
    // statements...
}
```

`Function main()` blocks are automatically unwrapped into the program body.

### Variable Declarations

```ys
var x = 10;              // Mutable, type inferred Int
var y: Float = 3.14;     // Mutable, explicit type
let z = 42;              // Immutable binding
const PI = 3.14159;      // Compile-time constant (same as let currently)
var w;                   // Default-initialized Int(0)
var s: String;           // Default-initialized String("")
```

- `var` — mutable (reassignable via `=` or `+=`/`-=` etc.)
- `let` — immutable
- `const` — compile-time constant

### Assignment

```ys
x = 15;                  // Reassign var
```

### Control Flow

```ys
// If/Else
if (condition) {
    // then
} else if (other) {
    // else if
} else {
    // else
}

// While
while (cond) {
    // body
}

// Numeric Loop
Loop(i from 0 to 10) {
    Print(i);            // 0, 1, 2, ..., 10 (inclusive both ends)
}

// For (iterable — parser struct, codegen TBD)
For(item in collection) {
    // body
}
```

### Functions

```ys
// Named function
Function Add(a: Int, b: Int) -> Int {
    Return a + b;
}

// No return type (infers from body or defaults to Void)
Function Greet(name: String) {
    PrintLine(name);
}

// Async (parser flag, codegen TBD)
async Function Fetch(url: String) -> String {
    Return await HttpGet(url);
}

// Differentiable (autodiff — parser flag, codegen TBD)
differentiable Function Loss(pred: Float, target: Float) -> Float {
    Return (pred - target) * (pred - target);
}
```

### Return

```ys
Return;             // return void
Return expr;        // return value
```

### Expressions

#### Binary Operators (precedence order)

| Level | Ops | Assoc | Description |
|-------|-----|-------|-------------|
| 1 | `!` `-` (unary) | Right | Logical NOT, numeric negation |
| 2 | `*` `/` `%` | Left | Multiply, divide, modulo |
| 3 | `+` `-` | Left | Add, subtract / string concat |
| 4 | `==` `!=` `<` `>` `<=` `>=` | Left | Comparison |
| 5 | `&&` | Left | Logical AND |
| 6 | `\|\|` | Left | Logical OR |
| 7 | `=` | Right | Assignment |

#### Unary Operators

```ys
-x              // Numeric negation
!flag           // Logical NOT
```

#### Function Call

```ys
MyFunc(arg1, arg2)
```

#### Method Call

```ys
object.Method(arg1, arg2)
```

#### Field Access

```ys
object.field
```

#### Index

```ys
array[index]
```

#### Block Expressions

```ys
{
    var x = 1;
    var y = 2;
    x + y   // block evaluates to this value
}
```

### Literals

```ys
42              // Int
3.14            // Float
true            // Bool
false           // Bool
"hello"         // String
null            // Null unit
[1, 2, 3]       // Array
```

### Comments

```ys
// Line comment
/* Block comment */
/* Nested /* block */ comments */
```

### Type Annotations

```ys
var x: Int = 5;
var y: Float = 3.14;
var z: Bool = true;
var s: String = "hi";
var t: _ = 10;              // Infer _
Function Add(a: Int, b: Int) -> Int { ... }
```

### Lambda Expressions

```ys
Function(x) { Return x * x; }
```

---

## Commands

### Main CLI: `oys`

```
Usage: oys <COMMAND>

Commands:
  build      Compile .ys to executable
  run        Compile and run
  test       Compile and run as test
  new        Create new project
  pack       Package management
  easy-debug Show transpiler output for .yse files
```

### `oys build`

```
oys build [options] <file.ys>

Options:
  -t, --target <target>    Target platform (native, wasm, gpu, game, kernel, server, desktop, mobile)
  -o, --output <path>      Output file path
  -L, --log-level <level>  Log level (error, warn, info, debug, trace)
  -e, --easy               Treat input as Y# Easy (.yse)
```

- Auto-detects `.yse` files by extension.
- Default target: `native` → produces `.exe`.
- Default output: `output.exe` (native), `output.wasm`, `output.spv`, `output.ysg`, `output.o`.
- Log levels control tracing from the build pipeline.

### `oys run`

```
oys run [options] <file.ys>

Options:
  -t, --target <target>    Target platform
  -e, --easy               Treat input as Y# Easy
```

Builds then executes the compiled binary immediately.

### `oys test`

```
oys test <file.ys|directory>
```

- Accepts a `.ys` file or directory.
- If directory, finds all `.ys` and `.yse` files.
- Each file is compiled and executed; exit code 0 = pass, non-zero = fail.
- Reports summary: `N passed, N failed, N total`.

### `oys new`

```
oys new <name>
```

Creates a new project directory with:
- `main.ys` — Hello World program
- `oy.toml` — project config

### `oys pack`

```
oys pack add <package>       Install a package
oys pack remove <package>    Remove a package
oys pack publish             Publish package
```

### `oys easy-debug`

```
oys easy-debug <file.yse>
```

Transpiles a `.yse` file to standard Y# and prints the result to stdout (no compilation).

### Alternative Entry: `yo`

An alternative CLI entry point with the same commands as `oys`.

### Targets

| Target | Extension | Description | Backend |
|--------|-----------|-------------|---------|
| `native` | `.exe` | Native Windows executable | C → GCC |
| `wasm` | `.wasm` | WebAssembly module | C backend |
| `gpu` / `ai` | `.spv` | SPIR-V 1.6 compute shader | GPU → SPIR-V |
| `game` | `.ysg` | Game engine C++20 output | Game C++ |
| `kernel` | `.o` | Kernel object | C backend |
| `server` | `.exe` | Server-optimized native | C backend |
| `desktop` | `.exe` | Desktop-optimized native | C backend |
| `mobile` | `.exe` | Mobile-optimized native | C backend |

---

## Runtime Functions

All built-in functions auto-registered in the type checker. String parameters/returns use an internal 64KB global buffer `_ys_retbuf`. Strings are passed as `int64_t` pointer casts.

### I/O

| Function | Signature | Description |
|----------|-----------|-------------|
| `Print(any)` | `T → Void` | Print value as string (polymorphic) |
| `PrintLine(any)` | `T → Void` | Print value + newline (polymorphic) |
| `ReadLine()` | `→ String` | Read line from stdin |
| `ReadInt()` | `→ Int` | Read integer from stdin |
| `ReadFloat()` | `→ Float` | Read float from stdin |
| `ReadAllText(path)` | `String → String` | Read entire file |
| `WriteAllText(path, data)` | `String, String → Void` | Write text to file |
| `AppendAllText(path, data)` | `String, String → Void` | Append text to file |
| `FileExists(path)` | `String → Bool` | Check if file exists |
| `FileDelete(path)` | `String → Void` | Delete file |
| `FileCopy(src, dst)` | `String, String → Void` | Copy file |
| `FileMove(src, dst)` | `String, String → Void` | Move/rename file |
| `FileSize(path)` | `String → Int` | Get file size in bytes |

### Directory

| Function | Signature | Description |
|----------|-----------|-------------|
| `DirCreate(path)` | `String → Void` | Create directory |
| `DirDelete(path)` | `String → Void` | Remove directory |
| `DirExists(path)` | `String → Bool` | Check if directory exists |
| `DirList(path)` | `String → String` | List directory (newline-separated) |
| `GetCurrentDir()` | `→ String` | Get current working directory |
| `SetCurrentDir(path)` | `String → Void` | Set current working directory |

### Console

| Function | Signature | Description |
|----------|-----------|-------------|
| `ClearScreen()` | `→ Void` | Clear console (`cls`) |
| `CursorPos(x, y)` | `Int, Int → Void` | Set cursor position (ANSI) |
| `GetCursorX()` | `→ Int` | Get cursor X (always 0 on Windows) |
| `GetCursorY()` | `→ Int` | Get cursor Y (always 0 on Windows) |
| `SetColor(code)` | `Int → Void` | Set ANSI color code |
| `ReadKey()` | `→ Int` | Read single keypress (no echo) |

### System

| Function | Signature | Description |
|----------|-----------|-------------|
| `ExitF(code)` | `Int → Void` | Exit process with code |
| `SleepF(ms)` | `Int → Void` | Sleep for N milliseconds |
| `Exec(cmd)` | `String → Int` | Execute shell command, return exit code |
| `ExecOutput(cmd)` | `String → String` | Execute command, capture stdout |
| `GetEnv(name)` | `String → String` | Get environment variable |
| `SetEnv(name, value)` | `String, String → Void` | Set environment variable |
| `GetOS()` | `→ String` | Get OS name ("Windows") |
| `GetPID()` | `→ Int` | Get process ID |
| `GetUserName()` | `→ String` | Get current user name |
| `GetHostName()` | `→ String` | Get machine hostname |
| `GetCPUCount()` | `→ Int` | Get number of CPU cores |

### Time

| Function | Signature | Description |
|----------|-----------|-------------|
| `NowUnix()` | `→ Int` | Unix timestamp (seconds) |
| `NowMillis()` | `→ Int` | Millisecond tick count |
| `NowString()` | `→ String` | Current datetime as `"YYYY-MM-DD HH:MM:SS"` |
| `DateString()` | `→ String` | Current date as `"YYYY-MM-DD"` |
| `TimeString()` | `→ String` | Current time as `"HH:MM:SS"` |
| `Year()` | `→ Int` | Current year |
| `Month()` | `→ Int` | Current month (1-12) |
| `Day()` | `→ Int` | Current day (1-31) |
| `Hour()` | `→ Int` | Current hour (0-23) |
| `Minute()` | `→ Int` | Current minute (0-59) |
| `Second()` | `→ Int` | Current second (0-59) |

### String Manipulation

| Function | Signature | Description |
|----------|-----------|-------------|
| `StringLen(s)` | `String → Int` | Get string length |
| `StringSub(s, start, len)` | `String, Int, Int → String` | Extract substring |
| `StringSplit(s, delim)` | `String, String → String` | Split by delimiter (newline-separated result) |
| `StringContains(s, pat)` | `String, String → Bool` | Check if contains substring |
| `StringReplace(s, from, to)` | `String, String, String → String` | Replace all occurrences |
| `StringTrim(s)` | `String → String` | Trim both sides |
| `StringTrimLeft(s)` | `String → String` | Trim leading whitespace |
| `StringTrimRight(s)` | `String → String` | Trim trailing whitespace |
| `StringToUpper(s)` | `String → String` | Convert to uppercase |
| `StringToLower(s)` | `String → String` | Convert to lowercase |
| `StringStartsWith(s, prefix)` | `String, String → Bool` | Check prefix |
| `StringEndsWith(s, suffix)` | `String, String → Bool` | Check suffix |
| `StringAt(s, idx)` | `String, Int → String` | Get character at index |
| `StringPadLeft(s, total, pad)` | `String, Int, String → String` | Left-pad string |
| `StringPadRight(s, total, pad)` | `String, Int, String → String` | Right-pad string |

### Conversion

| Function | Signature | Description |
|----------|-----------|-------------|
| `ToInt(v)` | `Int → Int` | Identity (type marker) |
| `ToFloat(v)` | `Float → Float` | Identity (type marker) |
| `ToString(v)` | `String → String` | Identity (type marker) |
| `ParseInt(s)` | `String → Int` | Parse string to Int |
| `ParseFloat(s)` | `String → Float` | Parse string to Float |
| `Format(fmt, arg)` | `String, String → String` | Format string |
| `IntToStr(v)` | `Int → String` | Int to string |
| `FloatToStr(v)` | `Float → String` | Float to string |
| `BoolToStr(v)` | `Bool → String` | Bool to string |
| `StrToInt(s)` | `String → Int` | String to Int (alias for ParseInt) |
| `StrToFloat(s)` | `String → Float` | String to Float (alias for ParseFloat) |
| `CharCode(s)` | `String → Int` | Get ASCII code of first character |
| `CodeChar(c)` | `Int → String` | Character from ASCII code |

### Math

| Function | Signature | Description |
|----------|-----------|-------------|
| `Abs(x)` | `Int → Int` | Absolute value (Int) |
| `AbsF(x)` | `Float → Float` | Absolute value (Float) |
| `Min(a, b)` | `Int, Int → Int` | Minimum (Int) |
| `MinF(a, b)` | `Float, Float → Float` | Minimum (Float) |
| `Max(a, b)` | `Int, Int → Int` | Maximum (Int) |
| `MaxF(a, b)` | `Float, Float → Float` | Maximum (Float) |
| `Clamp(v, lo, hi)` | `Int, Int, Int → Int` | Clamp value (Int) |
| `ClampF(v, lo, hi)` | `Float, Float, Float → Float` | Clamp value (Float) |
| `Sin(x)` | `Float → Float` | Sine (radians) |
| `Cos(x)` | `Float → Float` | Cosine (radians) |
| `Tan(x)` | `Float → Float` | Tangent (radians) |
| `Asin(x)` | `Float → Float` | Arc sine |
| `Acos(x)` | `Float → Float` | Arc cosine |
| `Atan(x)` | `Float → Float` | Arc tangent |
| `Atan2(y, x)` | `Float, Float → Float` | Arc tangent (two-argument) |
| `Sqrt(x)` | `Float → Float` | Square root |
| `Pow(x, y)` | `Float, Float → Float` | Power (x^y) |
| `Exp(x)` | `Float → Float` | Exponential (e^x) |
| `Log(x)` | `Float → Float` | Natural logarithm |
| `Log2(x)` | `Float → Float` | Base-2 logarithm |
| `Log10(x)` | `Float → Float` | Base-10 logarithm |
| `Floor(x)` | `Float → Int` | Round down |
| `Ceil(x)` | `Float → Int` | Round up |
| `Round(x)` | `Float → Int` | Round to nearest |
| `Trunc(x)` | `Float → Int` | Truncate |
| `Frac(x)` | `Float → Float` | Fractional part |
| `Sign(x)` | `Int → Int` | Sign (-1, 0, 1) |
| `SignF(x)` | `Float → Float` | Sign (-1.0, 0.0, 1.0) |
| `Lerp(a, b, t)` | `Float, Float, Float → Float` | Linear interpolation |
| `Random()` | `→ Float` | Random float [0, 1) |
| `RandomRange(min, max)` | `Float, Float → Float` | Random float in range |
| `RandomInt(min, max)` | `Int, Int → Int` | Random integer in range [min, max] |
| `SeedRandom(seed)` | `Int → Void` | Seed the RNG |
| `DegToRad(d)` | `Float → Float` | Degrees to radians |
| `RadToDeg(r)` | `Float → Float` | Radians to degrees |
| `Hypot(x, y)` | `Float, Float → Float` | Euclidean distance sqrt(x² + y²) |

### Memory / Advanced

| Function | Signature | Description |
|----------|-----------|-------------|
| `MemoryAddress(p)` | `Int → Int` | Return address as Int |
| `MemorySize()` | `→ Int` | Get heap size (always 0) |
| `StackAlloc(size)` | `Int → Int` | Allocate memory (malloc) |
| `StackFree(ptr)` | `Int → Void` | Free memory (free) |
| `CopyMem_(dst, src, n)` | `Int, Int, Int → Void` | Copy memory (memmove) |
| `CompareMemory(a, b, n)` | `Int, Int, Int → Bool` | Compare memory blocks |
| `SetMemory(ptr, val, n)` | `Int, Int, Int → Void` | Set memory (memset) |

### Process

| Function | Signature | Description |
|----------|-----------|-------------|
| `RunProcess(path, args)` | `String, String → Int` | Launch process, return exit code |
| `KillProcess(pid)` | `Int → Void` | Kill process (stub) |
| `ProcessExists(pid)` | `Int → Bool` | Check if process exists (stub) |
| `WaitProcess(pid)` | `Int → Int` | Wait for process (stub) |

### Network

| Function | Signature | Description |
|----------|-----------|-------------|
| `HttpGet(url)` | `String → String` | HTTP GET request (via PowerShell) |
| `HttpPost(url, data)` | `String, String → String` | HTTP POST request (via PowerShell) |
| `HttpGetJson(url)` | `String → String` | HTTP GET JSON |
| `HttpPostJson(url, data)` | `String, String → String` | HTTP POST JSON |
| `DownloadFile(url, path)` | `String, String → Bool` | Download file (via PowerShell) |
| `PingHost(host)` | `String → Bool` | Ping a host |
| `ResolveHost(host)` | `String → String` | DNS resolve to IP |

### Type Introspection

| Function | Signature | Description |
|----------|-----------|-------------|
| `TypeOf(v)` | `Int → String` | Type name (stub) |
| `IsInt(v)` | `Int → Bool` | Always true |
| `IsFloat(v)` | `Float → Bool` | Always true |
| `IsString(s)` | `String → Bool` | Always true (if non-null) |
| `IsBool(v)` | `Bool → Bool` | Always true |

---

## Error Reference

### Parse Errors (from grammar.rs / parser)

Errors from the nom-based parser carry an offset and a message.

| Error | Cause | Fix |
|-------|-------|-----|
| `expected letter` | Identifier starts with a non-letter, non-underscore character | Start identifiers with `a-z`, `A-Z`, or `_` |
| `expected digit` | Expected a digit in numeric literal | Ensure digit is present |
| `expected expression` | Valid expression not found at current position | Check syntax — missing value, stray operator, empty parens |
| `expected ')'` | Missing closing parenthesis | Add `)` |
| `expected '}'` | Missing closing brace | Add `}` or check indentation |
| `expected ';'` | Missing semicolon | Add `;` at end of statement |
| `expected ','` | Missing comma in function arguments or parameters | Add `,` between elements |
| `Incomplete input` | Input ended mid-parse | Complete the partial construct |
| `nom error: ...` | Generic nom combinator failure | Review syntax near the reported offset |

Parse error output format:
```
Parse error: expected expression (at offset 42, near "...snippet...")
```

### Type Errors (from infer.rs / unify.rs)

Type errors occur during Hindley-Milner type inference/unification.

| Error | Cause | Fix |
|-------|-------|-----|
| `Undefined variable: X` | Variable `X` not declared in scope | Declare with `var`, `let`, or check spelling |
| `Left operand must be numeric or string, got ...` | `+` on non-numeric, non-string type | Ensure left operand is Int, Float, or String |
| `Right operand must be numeric or string, got ...` | `+` on non-numeric, non-string type | Ensure right operand is Int, Float, or String |
| `Left operand must be numeric, got ...` | `-`, `*`, `/`, `%` on non-numeric type | Ensure operand is Int or Float |
| `Right operand must be numeric, got ...` | `-`, `*`, `/`, `%` on non-numeric type | Ensure operand is Int or Float |
| `Cannot negate ...` | Unary `-` on non-numeric type | Only negate Int or Float values |
| `Function X expects N arguments, got M` | Wrong argument count for function `X` | Pass exactly N arguments |
| `Function expects N arguments, got M` | Wrong argument count for generic function call | Pass exactly N arguments |
| `{:?} is not callable` | Attempted to call a non-function value | Ensure the callee is a function |
| `Type mismatch: A vs B` | Two types don't unify (e.g., `Int` vs `String`) | Make types consistent |
| `Occurs check failed: type variable N appears in ...` | Infinite type (recursive without explicit type) | Add explicit type annotation |
| `Tensor dimension mismatch: N vs M` | Tensor unification with different dimensions | Match tensor dimensions |
| `Function parameter count mismatch: N vs M` | Function type parameter count mismatch | Match function signatures |
| `Generic type mismatch: A vs B` | Generic type name mismatch | Use the same generic type |
| `Generic parameter count mismatch` | Wrong number of generic parameters | Match generic parameter count |
| `Custom type mismatch: A vs B` | Custom/Entity type name mismatch | Use the correct type name |
| `Custom type parameter count mismatch` | Wrong number of custom type parameters | Match parameter count |

Type error output:
```
type error: Undefined variable: xyz
```

### Build Errors (from session.rs)

| Error | Cause | Fix |
|-------|-------|-----|
| `cannot read source 'file': ...` | File not found or permission denied | Check path and file permissions |
| `invalid target 'X': unknown target 'X'` | Unrecognized `--target` value | Use one of: native, wasm, gpu, game, kernel, server, desktop, mobile |
| `lexical analysis failed` | Lexer encountered an invalid token | Check for illegal characters or malformed literals |
| `parsing failed: ...` | Parser error (see Parse Errors above) | Fix syntax near reported location |
| `type checking failed` | Type error (see Type Errors above) | Fix type issues |
| `AST conversion failed: ...` | HIR conversion error | Likely a compiler bug — report it |
| `HIR lowering failed: ...` | HIR lowering error | Likely a compiler bug — report it |
| `codegen failed: ...` | Code generation failure (e.g., GCC not found) | Install MinGW GCC, check PATH |
| `gcc: ...` | GCC compilation/link error | Fix C syntax issues (rare — usually a compiler bug) |

---

## Built-in Type System

### Primitive Types

| Type | Description | Size | Default Value |
|------|-------------|------|---------------|
| `Int` | Signed 64-bit integer (i64) | 8 B | `0` |
| `Float` | IEEE 754 double-precision (f64) | 8 B | `0.0` |
| `Bool` | Boolean | 1 B | `false` |
| `String` | UTF-8 string pointer (via `_ys_retbuf`) | 8 B ptr | `""` |
| `Null` | Unit type (void value) | 0 B | `null` |
| `Void` | No return type | 0 B | N/A |
| `_` (Infer) | Type inference placeholder | — | — |

### Compound Types

| Type | Description |
|------|-------------|
| `Tensor<T, N>` | N-dimensional tensor with element type T |
| `Function(params...) -> ret` | Function type signature |
| `Optional<T>` | Optional value of type T |
| `Array<T>` | Generic array type |
| `Custom(Name, params...)` | User-defined type (Entity, Component, etc.) |
| `Entity(Name)` | Entity type |

### Type Inference Rules

1. **Integer literals** → `Int`
2. **Float literals** → `Float`
3. **String literals** → `String`
4. **Bool literals** → `Bool`
5. **Binary `+`**: If either operand is `String`, both → `String` (concat). Else, if either is `Float`, result → `Float`. Otherwise → `Int`.
6. **Binary `-`, `*`, `/`, `%`**: Both must be numeric. If either is `Float`, result → `Float`. Otherwise → `Int`.
7. **Comparisons `==`, `!=`, `<`, `>`, `<=`, `>=`**: Unify operands, result → `Bool`.
8. **Logical `&&`, `||`**: Both must be `Bool`, result → `Bool`.
9. **Unary `-`**: Operand must be numeric, result is same type.
10. **Unary `!`**: Operand must be `Bool`, result → `Bool`.
11. **Loop variables**: Loop variable is bound as `Int`.
12. **If/else**: Both branches must unify to the same type.
13. **Blocks**: Evaluate to the type of the last expression.

### Type Annotations

```ys
var x: Int = 5;
var y: _ = 10;           // Infer (same as omitting type)
Function Add(a: Int, b: Float) -> Float { Return a + b; }
```

---

## ECS — Entity Component System

### Entity

Declares a named entity with inline component data:

```ys
Entity Player {
    Transform { x: 0, y: 0, z: 0, rotation: 0, scale: 1 }
    RigidBody { velocity: 0, mass: 1, drag: 0.1, useGravity: true }
}
```

### Component

Defines a component type with typed fields:

```ys
Component Transform {
    x: Float, y: Float, z: Float, rotation: Float, scale: Float
}
```

### System

Defines a system that queries entities by component:

```ys
System Movement(Transform) {
    ForEach(Transform, Function(entity) {
        // per-frame logic
    });
}
```

ECS is parsed into the AST (EntityDef, ComponentDef, SystemDef nodes) and lowered to HIR. The Game C++ backend emits archetype-based SoA iteration.

---

## Actor Model

```ys
Actor Counter {
    On(Increment) {
        State<Int> count;
        count = count + 1;
    }
    On(GetValue) {
        State<Int> count;
        Reply(count);
    }
}
```

- Actors have named `On(event)` handlers.
- `State<T> name` declares typed persistent state.
- `Reply(value)` sends a response.
- Parsed into ActorDef / OnHandler AST nodes.

---

## View & State

### View

Declares a UI view block:

```ys
View {
    // children
}
```

### State

Declares a typed reactive state variable:

```ys
State<Int> counter = 0;
State<String> name;
```

---

## Compiler Pipeline

```
.ys / .yse source
    → Easy Transpiler (indentation → braces)
    → Lexer (logos tokenizer)
    → Parser (nom combinator → AstArena, 38+ node types)
    → Type Checker (Hindley-Milner unification, 19 type variants)
    → HIR Lowerer (27+ high-level IR node types)
    → MIR Lowerer (14+ instructions, 4 terminators, CFG)
    → MIR Optimizer (const_fold → loop_opt → vectorize → reorder)
    → Codegen Dispatcher
        → C Backend (C99 → GCC → .exe)
        → Game Backend (C++20 → .ysg)
        → GPU Backend (SPIR-V 1.6 binary → .spv)
    → output
```

### MIR Optimizer Passes

| Pass | Function |
|------|----------|
| Constant Folding | Evaluates constant expressions at compile time |
| Loop Optimization (LICM) | Loop invariant code motion, induction variable analysis |
| Auto-Vectorization | Detects SIMD-izable loops (contiguous access, stride-1) |
| Block Reordering | Hot blocks contiguous, cold blocks to end (I-cache) |
| Inlining | Aggressive inlining of small functions |

### MIR Instruction Set

| Instruction | Description |
|-------------|-------------|
| `Alloca` | Stack allocation |
| `Load` | Load SSA value |
| `Store` | Store SSA value |
| `Binary` | Binary arithmetic/logic |
| `Unary` | Unary negation/not |
| `Call` | Function call |
| `IntLiteral` | Integer constant |
| `FloatLiteral` | Float constant |
| `StringLiteral` | String constant |
| `BoolLiteral` | Boolean constant |
| `Phi` | SSA phi node |
| `Param` | Load function parameter by index |
| `Print` | Polymorphic print intrinsic |
| `VectorHint` | SIMD vectorization hint |
| `InlineHint` | Inline hint |

### MIR Terminators

| Terminator | Description |
|------------|-------------|
| `Branch(block)` | Unconditional jump |
| `CondBranch { cond, true, false }` | Conditional branch |
| `Return(value)` | Return from function |
| `Unreachable` | Unreachable code |

---

## Codegen Backends

### C Backend (default for native/server/desktop/mobile/kernel/wasm)

- Converts MIR → C99 via `CMirModule` intermediate representation.
- Emits `#include` headers for Win32, math, string, socket, console I/O.
- Each MIR instruction maps to C statements:
  - `Alloca` → local variable declaration
  - `Binary` → `dest = lhs op rhs;`
  - `Call` → `dest = name(args);`
  - `Print` → `_ys_print_str(expr)` / `_ys_print_int(expr)` / `_ys_print_float(expr)`
  - `IntLiteral` → `dest = 42LL;`
  - `FloatLiteral` → `dest = 3.14;`
- String literals emitted as `const char _s0[] = "...";` globals.
- Each function's CFG blocks become C labels with goto branching.
- Compiles with `gcc -O2 -std=c99 -o output.exe output.c -lws2_32`.
- Searches for MinGW GCC in common Windows locations (Chocolatey, MSYS2, MinGW).

### Game Backend (C++20)

- Converts MIR → `GameMirModule` with game-specific types:
  - `Vec4` (oy::float4), `Mat4` (oy::float4x4), `Quat` (oy::Quat)
- Emits C++20 code with:
  - ECS archetype queries: `auto q = world.query<Transform, RigidBody>();`
  - Render passes: `render_pass("shadow", "shadow_shader", {targets...});`
  - Physics steps: `physics_step(dt);`
- Generates `main()` with game loop: `engine.init()` → `while (engine.running())` → `engine.shutdown()`.
- Includes `<oy_runtime.h>` header.
- Output: `.ysg` file.

### GPU Backend (SPIR-V 1.6)

- Converts MIR → `GpuMirModule` with GPU-specific ops:
  - `FAdd`, `FSub`, `FMul`, `FDiv` — float arithmetic
  - `Vec4Splat`, `Vec4Add`, `Vec4Mul`, `Vec4Dot` — vector ops
  - `Mat4Mul` — matrix multiply
  - `GlobalLoad`/`GlobalStore` — buffer access
  - `TensorRead`/`TensorWrite` — tensor element access
  - `Barrier` — workgroup synchronization
  - `WorkgroupSize` — local size hint
- Emits SPIR-V 1.6 binary via `SpvBuilder`:
  - Capabilities: Shader, Matrix, Float64, Int64, Float16Buffer
  - GLSL.std.450 extended instruction set
  - Compute shader entry point with `gl_GlobalInvocationID`
  - `OpControlBarrier` for workgroup sync
  - Default workgroup size: (64, 1, 1)
- Output: `.spv` binary file.

### WASM / LLVM

- `wasm` target routes through C backend (compile to C, `requires_wasm()` flag exists for future Emscripten/LLVM WASM toolchain).
- `kernel` target routes through C backend (produces `.o` object file).
- LLVM backend infrastructure reference exists (`requires_llvm()` returns `true` for native/server/desktop/mobile/kernel targets), but current codegen goes through C→GCC.

---

## Standard Library

| Module | Description |
|--------|-------------|
| `core/io.ys` | Print, PrintLine, ReadLine, File I/O, Format, ToString |
| `core/math.ys` | Abs, Min, Max, Sin, Cos, Sqrt, Pow, Random, RandomRange |
| `core/collections.ys` | List, Map, StringSplit/Contains/Replace |
| `ai/nn.ys` | Sequential, DenseLayer, ConvLayer, Optimizers |
| `ai/tensor.ys` | TensorCreate, MatMul, Conv2d, Relu, Softmax, MSE |
| `game/ecs.ys` | CreateEntity, AddComponent, Query, ForEach, Emit/On, Raycast |
| `game/physics.ys` | Vec3, Quat, Mat4 operations, collision detection |
| `server/http.ys` | Serve, Request, Response |
| `web/dom.ys` | QuerySelector, CreateElement, Events, Router, State |

---

## Installation

### Windows Installer (recommended)

Download `YSharp-v8.0.5-windows-x64.exe` from the [Releases page](https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases) and double-click.

The NSIS installer lets you:
- Choose installation directory (default: `C:\Program Files\YSharp`)
- Optionally add `oys`/`yo` to the system PATH
- Associate `.ys`/`.yse` files (double-click to build & run, like Python)
- Create Start Menu shortcuts
- Cleanly uninstall from Control Panel

### npm

```bash
npm install -g ys-lang
oys build myprogram.ys
```

### From Source

Requires Rust 1.96+ and MinGW GCC 15.2+.

```bash
git clone https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp.git
cd YSharp-YSharp
cargo build --release --target x86_64-pc-windows-gnu
```

---

## License

MIT — Y# v8.0.5 "Oyster Shell"
