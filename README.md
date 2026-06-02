# Y# (YSharp) v8.0.0 — Oyster Shell

> *Y# (pronounced "why-sharp" or "oyster") is a modern systems programming language for games, AI, and high-performance computing.*

[![GitHub release](https://img.shields.io/github/v/release/ouzlifaneyassine1-dot/YSharp-YSharp)](https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp/releases/tag/v8.0.0)

---

## Description

Y# is a compiled, statically-typed systems language with:
- **Built-in ECS architecture** (Entity/Component/System) for game development
- **GPU compute kernel compilation** (SPIR-V 1.6 backend)
- **Automatic differentiation** for ML/AI
- **Actor model** for concurrent systems
- **Tensor operations** and neural network primitives
- **Multiple codegen backends** (Native C, Game C++, GPU SPIR-V, WASM, LLVM)

### V8 Feature Set

| Category | Status |
|----------|--------|
| Polymorphic Print/PrintLine | ✓ |
| Variables (var/let/const) + type inference | ✓ |
| Conditionals (if/else) | ✓ |
| Numeric Loop (Loop i from x to y) | ✓ |
| While loops | ✓ |
| Functions with params + return types | ✓ |
| Binary/unary expressions | ✓ |
| Comments (//, /* */) | ✓ |
| Full compiler pipeline | ✓ |
| Windows self-extracting installer | ✓ |
| ECS (Entity/Component/System) | Parser + HIR |
| Actor model | Parser + HIR |
| GPU compute (SPIR-V backend) | ✓ |
| MIR optimizer (const fold, LICM, vectorize, reorder) | ✓ |
| npm distribution package | ✓ |

---

## Quick Start

### Hello World

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

### Print Polymorphism

```ys
Program PolyPrint {
    Print(42);                // "42"
    Print(3.14);              // "3.14"
    Print("hi");              // "hi"
    Print(true);              // "true"
    PrintLine("done");        // "done\n"
}
```

---

## Installation

### Windows Installer (recommended)

Download `ys-v8.0.0-windows-x64-installer.exe` and run it.

```
ys-v8.0.0-windows-x64-installer.exe /S        # Silent install
ys-v8.0.0-windows-x64-installer.exe /D=C:\ys  # Custom path
ys-v8.0.0-windows-x64-installer.exe /NOPATH   # Skip PATH
ys-v8.0.0-windows-x64-installer.exe /UNINSTALL # Uninstall
```

### npm

```bash
npm install ys-lang
npx ys-lang build myprogram.ys
```

### From Source

Requires Rust 1.96+ and MinGW GCC 15.2+.

```bash
git clone https://github.com/ouzlifaneyassine1-dot/YSharp-YSharp.git
cd YSharp-YSharp
cargo build --release --target x86_64-pc-windows-gnu
```

---

## CLI Usage

| Command | Description |
|---------|-------------|
| `oys build <file>` | Compile `.ys` to executable |
| `oys run <file>` | Compile and run |
| `oys test <file\|dir>` | Compile and run as test |
| `oys new <name>` | Create new project |
| `oys pack add <pkg>` | Install a package |
| `oys pack remove <pkg>` | Remove a package |
| `oys pack publish` | Publish package |

### Build Options

```
oys build [options] <file.ys>

Options:
  -t, --target <target>    Target platform (native, wasm, gpu, game, kernel, server, desktop, mobile)
  -o, --output <path>      Output file path
  -L, --log-level <level>  Log level (error, warn, info, debug, trace)
```

---

## Language Reference

### Types

| Type | Description | Size |
|------|-------------|------|
| `Int` | Signed 64-bit integer | 8 B |
| `Float` | IEEE 754 double | 8 B |
| `Bool` | Boolean | 1 B |
| `String` | UTF-8 string | 16 B |
| `Null` | Unit type | 0 B |
| `Void` | No return | 0 B |

### Variables

```ys
var x = 10;              // Mutable, type inferred
var y: Float = 3.14;     // Explicit type
let z = 42;              // Immutable
const PI = 3.14159;      // Compile-time constant
var w;                   // Default Int
```

### Control Flow

```ys
// If/Else
if (condition) { } else if (other) { } else { }

// While
while (condition) { }

// Numeric Loop
Loop(i from 0 to 10) { Print(i); }

// For (iterable)
For(item in collection) { }
```

### Functions

```ys
Function Add(a: Int, b: Int) -> Int {
    Return a + b;
}

// Async
async Function Fetch(url: String) -> String {
    Return await HttpGet(url);
}

// Differentiable (autodiff)
differentiable Function Loss(pred: Float, target: Float) -> Float {
    Return (pred - target) * (pred - target);
}
```

### Operators (precedence order)

| Level | Ops | Assoc |
|-------|-----|-------|
| 1 | `!` `-` (unary) | Right |
| 2 | `*` `/` `%` | Left |
| 3 | `+` `-` | Left |
| 4 | `==` `!=` `<` `>` `<=` `>=` | Left |
| 5 | `&&` | Left |
| 6 | `\|\|` | Left |
| 7 | `=` | Right |

---

## Compiler Pipeline

```
.ys source
    → Lexer (logos tokenizer)
    → Parser (nom combinator → AstArena, 38 node types)
    → Type Checker (Hindley-Milner unification, 14 type variants)
    → HIR Lower (27 high-level IR node types)
    → MIR Lower (14 instructions, 4 terminators, CFG)
    → MIR Optimizer (const_fold → loop_opt → vectorize → reorder)
    → Codegen (C / Game C++ / GPU SPIR-V / WASM / LLVM)
    → output.exe / .ysg / .spv
```

### MIR Optimizer Passes

| Pass | Function |
|------|----------|
| Constant Folding | Evaluates constant expressions at compile time |
| Loop Optimization (LICM) | Loop invariant code motion, induction variable analysis |
| Auto-Vectorization | Detects SIMD-izable loops (contiguous access, stride-1) |
| Block Reordering | Hot blocks contiguous, cold blocks to end (I-cache) |
| Inlining | Aggressive inlining of small functions |

---

## Standard Library

| Module | File | Contents |
|--------|------|----------|
| Core IO | `core/io.ys` | Print, PrintLine, ReadLine, File I/O, Format, ToString |
| Core Math | `core/math.ys` | Abs, Min, Max, Sin, Cos, Sqrt, Pow, Random, RandomRange |
| Core Collections | `core/collections.ys` | List, Map, StringSplit/Contains/Replace |
| AI/NN | `ai/nn.ys` | Sequential, DenseLayer, ConvLayer, Optimizers |
| AI/Tensor | `ai/tensor.ys` | TensorCreate, MatMul, Conv2d, Relu, Softmax, MSE |
| Game/ECS | `game/ecs.ys` | CreateEntity, AddComponent, Query, ForEach, Emit/On, Raycast |
| Game/Physics | `game/physics.ys` | Vec3, Quat, Mat4 operations, collision detection |
| Server/HTTP | `server/http.ys` | Serve, Request, Response |
| Web/DOM | `web/dom.ys` | QuerySelector, CreateElement, Events, Router, State |

---

## ECS — Entity Component System

```ys
Component Transform { x: Float, y: Float, z: Float, rotation: Float, scale: Float }

Entity Player {
    Transform { x: 0, y: 0, z: 0, rotation: 0, scale: 1 }
    RigidBody { velocity: Vec3(0,0,0), mass: 1, drag: 0.1, useGravity: true }
}

System Movement(Transform) {
    ForEach(Transform, Function(entity) {
        // per-frame logic
    });
}
```

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

## GPU Compute

```ys
differentiable Function MatMul(a: Tensor, b: Tensor) -> Tensor
```

Compiles to SPIR-V 1.6 compute shaders with workgroup parallelism, shared memory barriers, and vectorized math.

---

## License

MIT — Y# v8.0.0 "Oyster Shell"
