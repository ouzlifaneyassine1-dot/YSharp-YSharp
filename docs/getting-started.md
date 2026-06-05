# Getting Started with Y#

## Installation

### Windows Installer (recommended)

Download `YSharp-v8.0.2-windows-x64.exe` from the [Releases page](https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases) and double-click.

Choose installation directory and optionally add to PATH.

### via npm

```bash
npm install -g ys-lang
```

### From Source

```bash
git clone https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp.git
cd YSharp-YSharp
cargo build --release
```

---

## Hello World

### Easy syntax (`.yse`)

```yse
// hello.yse
fn Main
    println Hello, World!
```

Compile and run:
```bash
oys build hello.yse
./hello
```

### Standard syntax (`.ys`)

```ys
// hello.ys
Function Main() -> Void
{
    PrintLine("Hello, World!")
}
```

Build and run:
```bash
oys build hello.ys
./hello
```

---

## Quick Tour

### Variables

```ys
var x: Int = 10          // mutable
let name: String = "Y#"  // immutable
const PI: Float = 3.14   // compile-time constant
var y := 20              // type inference with :=
var z: _ = 30            // explicit infer (_)
```

### Loops

```ys
// while
var i: Int = 0
while i < 5
{
    PrintLine(i)
    i = i + 1
}

// Loop (infinite, with break)
Loop
{
    var line = ReadLine()
    if line == ""
        break
    PrintLine(line)
}

// For range
for i in 0..10
{
    PrintLine(i)
}
```

### Conditionals

```ys
var score: Int = 85

if score >= 90
    PrintLine("A")
else if score >= 80
    PrintLine("B")
else
    PrintLine("C")
```

### Functions

```ys
Function Add(a: Int, b: Int) -> Int
{
    return a + b
}

// Void return
Function Greet(name: String) -> Void
{
    PrintLine("Hello, " + name)
}
```

---

## Building and Running

| Command               | Description                        |
|-----------------------|------------------------------------|
| `oys build file.ys`   | Compile to native executable       |
| `oys run file.ys`     | Compile and run                    |
| `oys test`            | Run tests                          |
| `oys pack`            | Package project                    |
| `oys new my_project`  | Scaffold new project               |

### Flags

```bash
oys build file.ys --cpp    # compile as C++ (g++)
oys build file.ys -o out   # custom output name
oys build file.ys --opt 2  # optimization level 2
oys build file.ys -e       # easy transpile mode
```
