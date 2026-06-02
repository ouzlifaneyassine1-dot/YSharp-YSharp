# Y# v8.0.0 — Oyster Shell

Y# (pronounced "oyster") is a modern systems programming language with built-in ECS, GPU compute, and actor model support.

## Features (V8)

- Print polymorphism — `Print()` and `PrintLine()` accept any type
- Variables — `var`, `let`, `const` with type inference
- Conditionals — `if`/`else`
- Loops — `Loop(var from expr to expr) { body }`
- Multiple codegen backends (Native/C, Game/C++, GPU/SPIR-V)
- MIR optimizer (constant folding, LICM, auto-vectorization, block reordering)

## Installation

### Windows
```
ys.exe build myprogram.ys
```

### npm (Windows)
```
npx ys-lang build myprogram.ys
```

### From source
Requires Rust 1.96+ and MinGW GCC.
```
cargo build --release --target x86_64-pc-windows-gnu
```

## Quick Start
```
Program Hello {
    PrintLine("Hello from Y#!");
}
```

## Usage
```
oys.exe build <file.ys>       # Compile to executable
oys.exe run <file.ys>         # Compile and run
oys.exe new <name>            # Create new project
yo install <package>          # Install package
```

## License
MIT
