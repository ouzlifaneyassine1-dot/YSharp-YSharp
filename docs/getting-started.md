# Getting Started with Y#

## Installation

### via npm

```bash
npm install -g ysharp-lang
```

### via Installer

Download the latest installer from the [releases page](https://github.com/ysharp/releases).

### Build from Source

```bash
git clone https://github.com/ysharp/ysharp.git
cd ysharp
cargo build --release
# Binary at ./target/release/ys
```

---

## Hello World

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
ys build hello.ys
./hello
```

### Easy syntax (`.yse`)

```yse
// hello.yse
fn Main() -> Void
    PrintLine("Hello, World!")
```

Transpile then build:

```bash
ys easy-debug hello.yse      # see transpiled output
ys build hello.ys             # build the generated .ys file
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
| `ys build file.ys`    | Compile to native executable       |
| `ys run file.ys`      | Compile and run                    |
| `ys test`             | Run tests                          |
| `ys pack`             | Package project                    |
| `ys new my_project`   | Scaffold new project               |

### Flags

```bash
ys build file.ys --target game    # compile as game (C++→g++)
ys build file.ys -o my_game       # custom output name
ys build file.ys --opt 2          # optimization level 2
```
