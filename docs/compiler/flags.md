# CLI Flags Reference

## `--target` / `-t`

Select the compilation backend.

```bash
ys build file.ys --target native       # C → gcc (default)
ys build file.ys -t game               # C++ → g++
ys build file.ys -t gpu                # SPIR-V compute
ys build file.ys -t wasm               # WebAssembly
ys build file.ys -t llvm               # LLVM IR
```

---

## `--output` / `-o`

Specify the output filename.

```bash
ys build file.ys -o my_program         # produces my_program.exe
ys build file.ys --output game.exe     # custom name
```

---

## `--easy` / `-e`

Treat input as Easy syntax (`.yse`) without requiring the `.yse` extension.

```bash
ys build script.yse                    # auto-detected
ys build script.txt -e                 # force Easy mode
```

---

## `--link` / `-l`

Link an external library (passed to gcc/g++).

```bash
ys build file.ys -l m                  # link libm (math)
ys build file.ys -l pthread            # link pthreads
ys build file.ys -l sdl2               # link SDL2
ys build file.ys -l m -l pthread       # multiple libs
```

---

## `--cpp`

Use `g++` instead of `gcc` for the native target (same as `--target game` but without game-specific runtime).

```bash
ys build file.ys --cpp                 # compile with g++
```

---

## `--opt` / `-O`

Optimization level. Passed to the C/C++ compiler.

| Level | Description              |
|-------|--------------------------|
| `0`   | No optimization (debug)  |
| `1`   | Basic optimization       |
| `2`   | Standard optimization    |
| `3`   | Aggressive optimization  |
| `s`   | Optimize for size        |
| `z`   | Aggressive size opt      |

```bash
ys build file.ys -O 2                  # -O2 optimization
ys build file.ys --opt s               # optimize for size
ys build file.ys -O 3                  # highest perf
```

---

## `--log-level` / `-L`

Set logging verbosity.

| Level   | Description          |
|---------|----------------------|
| `error` | Errors only          |
| `warn`  | Errors + warnings    |
| `info`  | Normal (default)     |
| `debug` | Verbose diagnostics  |

```bash
ys build file.ys -L debug              # see all internal steps
ys build file.ys --log-level warn      # quiet output
```

---

## Subcommands

### `build`

```bash
ys build <file> [options]
```

Compile a source file to an executable.

### `run`

```bash
ys run <file> [options] [-- <args>...]
```

Compile and run immediately. Arguments after `--` are passed to the program.

```bash
ys run file.ys -- --verbose --count 5
```

### `test`

```bash
ys test [options]
```

Compile and run all test functions (functions annotated with `@test`).

### `pack`

```bash
ys pack [--output archive.tar.gz]
```

Package the project sources and config into a distributable archive.

### `new`

```bash
ys new <project_name>
```

Scaffold a new Y# project with standard directory layout.

```bash
ys new my_game
# Creates: my_game/
#   ├── src/
#   │   └── main.ys
#   ├── ys.json
#   └── README.md
```

### `easy-debug`

```bash
ys easy-debug <file.yse>
```

Transpile a `.yse` file to standard `.ys` and print to stdout. Does not compile.

```bash
ys easy-debug script.yse
```

---

## Examples

```bash
# Debug build
ys build hello.ys -o hello_debug -O 0 -L debug

# Game build with SDL2
ys build game.ys --target game -l sdl2 -o my_game

# Size-optimized release
ys build app.ys -O s -o app_small

# Run with args
ys run server.ys -- --port 8080

# Link multiple libs
ys build net.ys -l curl -l ssl -l crypto -o network_app
```
