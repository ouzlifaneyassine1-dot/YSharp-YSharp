# Compiler Architecture

## Build Pipeline

```
Source (.ys/.yse)
    │
    ▼
┌─────────────┐
│   Lexer     │  Tokenizes source into tokens (keywords, identifiers, literals, etc.)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  Builds AST (Abstract Syntax Tree) from token stream
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Type Check │  Resolves types, infers _, validates operations, reports type errors
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  HIR Lower  │  Lowers AST to HIR (High-level IR) — desugars loops, simplifies expressions
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  MIR Lower  │  Lowers HIR to MIR (Mid-level IR) — introduces explicit control flow, SSA form
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Codegen    │  Emits backend code (C, C++, SPIR-V, WASM, LLVM IR)
└─────────────┘
```

---

## Phase Details

### Lexer

Reads the source file character by character and produces tokens: `Function`, `if`, `while`, `{`, `}`, `42`, `"hello"`, `+`, etc. Handles string escaping, comment stripping, and whitespace.

### Parser

Consumes tokens and builds an AST. For `.yse` files, a transpiler step runs first to convert indentation-based blocks into brace-delimited `.ys` code.

### Type Checker

Walks the AST and assigns types to every expression. Infers types for `_` and `:=`. Reports type mismatches, undefined references, and invalid operations.

### HIR (High-level IR)

Desugars language constructs. `for` loops become `while` loops with counters. Compound operators (`+=`) expand. Pattern matching is resolved.

### MIR (Mid-level IR)

Introduces explicit control flow graphs, SSA (Static Single Assignment) form, and basic blocks. Optimizations run here: constant folding, dead code elimination, inlining.

### Codegen

Translates MIR to the target backend's output. Handles platform-specific ABI, calling conventions, and linkage.

---

## Backends

| Backend   | Language   | Compiler  | Output             | Use Case             |
|-----------|------------|-----------|--------------------|----------------------|
| Native    | C          | `gcc`     | Executable / .so   | General purpose      |
| Game      | C++        | `g++`     | Executable         | Game development     |
| GPU       | SPIR-V     | —         | .spv               | Compute shaders      |
| WASM      | WebAssembly| —         | .wasm              | Web/browser          |
| LLVM      | LLVM IR    | `llc`     | .o / executable    | Advanced optimization|

### Selecting a backend

```bash
ys build file.ys --target native   # default
ys build file.ys --target game     # C++ backend
ys build file.ys --target gpu      # SPIR-V kernels
ys build file.ys --target wasm     # WebAssembly
ys build file.ys --target llvm     # LLVM IR
```

---

## CLI Overview

```bash
ys <subcommand> [options] [file]
```

| Subcommand   | Description                        |
|--------------|------------------------------------|
| `build`      | Compile source to executable       |
| `run`        | Compile and run                    |
| `test`       | Compile and run tests              |
| `pack`       | Package project                    |
| `new`        | Scaffold a new project             |
| `easy-debug` | Transpile `.yse` to `.ys` (stdout) |
