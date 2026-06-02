# Y# (YSharp) v8.0.0 — Oyster Shell

> Y# (pronounced "why-sharp" or "oyster") is a modern systems programming language for games, AI, and high-performance computing.

---

## Description

Y# is a compiled, statically-typed systems language with:
- Built-in ECS architecture (Entity/Component/System)
- GPU compute kernel compilation (SPIR-V 1.6)
- Automatic differentiation for ML/AI
- Actor model for concurrent systems
- Tensor operations and neural network primitives
- Multiple codegen backends (Native C, Game C++, GPU SPIR-V, WASM, LLVM)

### V8 Feature Set

- Polymorphic Print/PrintLine
- Variables (var/let/const) + type inference
- Conditionals (if/else)
- Numeric Loop (Loop i from x to y)
- While loops
- Functions with params + return types
- Full compiler pipeline
- Windows self-extracting installer
- MIR optimizer (const fold, LICM, vectorize, reorder)

---

## Quick Start

```ys
Program Hello {
    Print("Hello, World!");
}
```

```bash
oys build hello.ys
./output.exe   # → Hello, World!
```

### Variables & Math

```ys
Program Math {
    var x = 10;
    var y = 20;
    PrintLine(x + y * 2);     // 50
    PrintLine((x + y) * 2);   // 60
}
```

### Loop + Conditionals

```ys
Program LoopTest {
    Loop(i from 1 to 5) {
        if (i % 2 == 0) {
            PrintLine(i);     // 2, 4
        } else {
            Print(i);         // 1, 3, 5
        }
    }
}
```

---

## Installation

### Windows Installer (recommended)

Run `ys-v8.0.0-windows-x64-installer.exe`

```
/S        Silent install
/D=C:\ys  Custom path
/NOPATH   Skip PATH modification
/UNINSTALL Uninstall
/?        Help
```

### From Source

Requires Rust 1.96+ and MinGW GCC 15.2+.

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

---

## CLI Usage

| Command | Description |
|---------|-------------|
| `oys build <file>` | Compile to executable |
| `oys run <file>` | Compile and run |
| `oys test <file\|dir>` | Test mode |
| `oys new <name>` | Create project |
| `oys pack add <pkg>` | Install package |

---

## Language Reference

### Types: Int, Float, Bool, String, Null, Void

### Variables
```ys
var x = 10;              // Mutable, inferred
var y: Float = 3.14;     // Explicit type
let z = 42;              // Immutable
const PI = 3.14159;      // Compile-time
```

### Control Flow
```ys
if (condition) { } else { }
while (condition) { }
Loop(i from 0 to 10) { Print(i); }
```

### Functions
```ys
Function Add(a: Int, b: Int) -> Int { Return a + b; }
async Function Fetch(url: String) -> String { Return await HttpGet(url); }
differentiable Function Loss(pred: Float, target: Float) -> Float { ... }
```

---

## Standard Library

| Module | Contents |
|--------|----------|
| Core IO | Print, PrintLine, ReadLine, File I/O, Format |
| Core Math | Abs, Sin, Cos, Sqrt, Pow, Random |
| Core Collections | List, Map, String utilities |
| AI/NN | Sequential, DenseLayer, ConvLayer |
| AI/Tensor | TensorCreate, MatMul, Conv2d, Relu |
| Game/ECS | CreateEntity, AddComponent, Query, ForEach |
| Game/Physics | Vec3, Quat, Mat4, collision |
| Server/HTTP | Serve, Request, Response |
| Web/DOM | QuerySelector, Events, Router |

---

## License

MIT — Y# v8.0.0 "Oyster Shell"
