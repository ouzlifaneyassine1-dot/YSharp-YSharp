# Standard Library Reference

All built-in functions organized by category.

---

## I/O

```ys
Function Print(value: ...) -> Void
```
Prints a value without a trailing newline.

```ys
Function PrintLine(value: ...) -> Void
```
Prints a value followed by a newline.

```ys
Function ReadLine() -> String
```
Reads a line from stdin. Returns `""` on EOF.

```ys
Function ReadInt() -> Int
```
Reads an integer from stdin. Returns `0` on invalid input.

```ys
Function ReadFloat() -> Float
```
Reads a float from stdin. Returns `0.0` on invalid input.

```ys
Function ReadAllText(path: String) -> String
```
Reads the entire file at `path` into a string.

```ys
Function WriteAllText(path: String, content: String) -> Void
```
Writes `content` to `path`, overwriting if it exists.

```ys
Function AppendAllText(path: String, content: String) -> Void
```
Appends `content` to `path`.

```ys
Function FileExists(path: String) -> Bool
```
Returns `true` if `path` exists and is a file.

```ys
Function FileDelete(path: String) -> Void
```
Deletes the file at `path`.

```ys
Function FileCopy(src: String, dst: String) -> Void
```
Copies `src` to `dst`.

```ys
Function FileMove(src: String, dst: String) -> Void
```
Moves `src` to `dst`.

```ys
Function FileSize(path: String) -> Int
```
Returns the file size in bytes.

```ys
Function FileReadLines(path: String) -> String[]
```
Returns an array of lines from the file.

---

## Directory

```ys
Function DirCreate(path: String) -> Void
```
Creates a directory and any missing parents.

```ys
Function DirDelete(path: String) -> Void
```
Deletes an empty directory.

```ys
Function DirExists(path: String) -> Bool
```
Returns `true` if `path` is a directory.

```ys
Function DirList(path: String) -> String[]
```
Returns file/directory names in `path`.

```ys
Function DirListFull(path: String) -> String[]
```
Returns full paths of entries in `path`.

```ys
Function DirCurrent() -> String
```
Returns the current working directory.

```ys
Function DirSet(path: String) -> Void
```
Changes the working directory to `path`.

---

## Console

```ys
Function ClearScreen() -> Void
```
Clears the terminal screen.

```ys
Function CursorPos(x: Int, y: Int) -> Void
```
Moves the cursor to column `x`, row `y`.

```ys
Function GetCursorX() -> Int
```
Returns the current cursor column.

```ys
Function GetCursorY() -> Int
```
Returns the current cursor row.

```ys
Function SetColor(foreground: String, background: String) -> Void
```
Sets terminal colors (e.g., `"red"`, `"green"`, `"blue"`, `"white"`, `"black"`).

```ys
Function ResetColor() -> Void
```
Resets terminal colors to defaults.

```ys
Function ReadKey() -> String
```
Reads a single key press. Returns the key as a string.

```ys
Function KeyPressed() -> Bool
```
Returns `true` if a key is in the input buffer.

```ys
Function SetTitle(title: String) -> Void
```
Sets the terminal window title.

---

## System

```ys
Function ExitF(code: Int) -> Void
```
Exits the process with code `code`.

```ys
Function SleepF(ms: Int) -> Void
```
Sleeps for `ms` milliseconds.

```ys
Function Exec(cmd: String) -> Int
```
Executes a shell command; returns the exit code.

```ys
Function ExecOutput(cmd: String) -> String
```
Executes a command and returns stdout as a string.

```ys
Function GetEnv(name: String) -> String
```
Returns the value of environment variable `name`, or `""` if not set.

```ys
Function SetEnv(name: String, value: String) -> Void
```
Sets environment variable `name` to `value`.

```ys
Function UnsetEnv(name: String) -> Void
```
Unsets environment variable `name`.

```ys
Function GetOS() -> String
```
Returns `"windows"`, `"linux"`, or `"macos"`.

```ys
Function GetPID() -> Int
```
Returns the current process ID.

```ys
Function GetArgs() -> String[]
```
Returns command-line arguments (excluding the program name).

```ys
Function GetArgCount() -> Int
```
Returns the number of command-line arguments.

```ys
Function GetExePath() -> String
```
Returns the full path of the current executable.

---

## Time

```ys
Function NowUnix() -> Int
```
Returns seconds since Unix epoch (1970-01-01 UTC).

```ys
Function NowMillis() -> Int
```
Returns milliseconds since Unix epoch.

```ys
Function NowString() -> String
```
Returns current time as `"2026-06-05T16:00:00Z"` (ISO 8601).

```ys
Function DateString() -> String
```
Returns current date as `"2026-06-05"`.

```ys
Function TimeString() -> String
```
Returns current time as `"16:00:00"`.

```ys
Function Sleep(seconds: Float) -> Void
```
Sleeps for `seconds` (supports fractional).

```ys
Function TickCount() -> Int
```
Returns milliseconds since program start.

---

## String

```ys
Function StringLen(s: String) -> Int
```
Returns the number of UTF-8 characters in `s`.

```ys
Function StringSub(s: String, start: Int, len: Int) -> String
```
Returns substring of `s` from `start` with length `len`.

```ys
Function StringSplit(s: String, delimiter: String) -> String[]
```
Splits `s` by `delimiter` into an array.

```ys
Function StringContains(s: String, substr: String) -> Bool
```
Returns `true` if `s` contains `substr`.

```ys
Function StringReplace(s: String, from: String, to: String, count: Int) -> String
```
Replaces up to `count` occurrences of `from` with `to`. Pass `-1` for all.

```ys
Function StringTrim(s: String) -> String
```
Removes leading and trailing whitespace.

```ys
Function StringTrimStart(s: String) -> String
```
Removes leading whitespace.

```ys
Function StringTrimEnd(s: String) -> String
```
Removes trailing whitespace.

```ys
Function StringToUpper(s: String) -> String
```
Converts to uppercase.

```ys
Function StringToLower(s: String) -> String
```
Converts to lowercase.

```ys
Function StringStartsWith(s: String, prefix: String) -> Bool
```
Returns `true` if `s` starts with `prefix`.

```ys
Function StringEndsWith(s: String, suffix: String) -> Bool
```
Returns `true` if `s` ends with `suffix`.

```ys
Function StringIndexOf(s: String, substr: String, start: Int) -> Int
```
Returns the index of `substr` in `s` starting at `start`, or `-1`.

```ys
Function StringReverse(s: String) -> String
```
Returns the reversed string.

```ys
Function StringRepeat(s: String, count: Int) -> String
```
Returns `s` repeated `count` times.

```ys
Function StringFormat(format: String, args: ...) -> String
```
Formats a string with positional arguments (e.g., `"Hello {0}"`).

```ys
Function StringIsEmpty(s: String) -> Bool
```
Returns `true` if `s` is `""`.

```ys
Function StringIsWhitespace(s: String) -> Bool
```
Returns `true` if `s` contains only whitespace.

```ys
Function StringCompare(a: String, b: String, ignoreCase: Bool) -> Int
```
Returns `-1`, `0`, or `1` for lexicographic comparison.

```ys
Function StringToChars(s: String) -> Int[]
```
Returns an array of Unicode code points.

---

## Conversion

```ys
Function ToInt(value: ...) -> Int
```
Converts a value to Int. Works with Float, String, Bool.

```ys
Function ToFloat(value: ...) -> Float
```
Converts a value to Float. Works with Int, String, Bool.

```ys
Function ToString(value: ...) -> String
```
Converts any value to its string representation.

```ys
Function ToBool(value: ...) -> Bool
```
Converts a value to Bool. `0`, `""`, `0.0` → `false`; everything else → `true`.

```ys
Function ParseInt(s: String) -> Int
```
Parses a string to Int. Returns `0` on failure.

```ys
Function ParseFloat(s: String) -> Float
```
Parses a string to Float. Returns `0.0` on failure.

```ys
Function ToHex(value: Int) -> String
```
Converts an Int to a hex string (e.g., `"FF"`).

```ys
Function FromHex(s: String) -> Int
```
Parses a hex string to Int (e.g., `"FF"` → `255`).

```ys
Function ToBin(value: Int) -> String
```
Converts an Int to a binary string (e.g., `"1010"`).

```ys
Function ToChar(code: Int) -> String
```
Converts a Unicode code point to a single-character string.

```ys
Function ToCodePoint(s: String) -> Int
```
Returns the Unicode code point of the first character in `s`.

---

## Math

```ys
Function Abs(value: ...) -> ...
```
Absolute value. Works on Int and Float.

```ys
Function Min(a: ..., b: ...) -> ...
```
Returns the smaller of two values. Works on Int and Float.

```ys
Function Max(a: ..., b: ...) -> ...
```
Returns the larger of two values. Works on Int and Float.

```ys
Function Clamp(value: ..., min: ..., max: ...) -> ...
```
Clamps `value` to `[min, max]`.

```ys
Function Sin(x: Float) -> Float
```
Trigonometric sine (radians).

```ys
Function Cos(x: Float) -> Float
```
Trigonometric cosine (radians).

```ys
Function Tan(x: Float) -> Float
```
Trigonometric tangent (radians).

```ys
Function ASin(x: Float) -> Float
```
Inverse sine.

```ys
Function ACos(x: Float) -> Float
```
Inverse cosine.

```ys
Function ATan(x: Float) -> Float
```
Inverse tangent.

```ys
Function ATan2(y: Float, x: Float) -> Float
```
Inverse tangent of `y/x`.

```ys
Function Sinh(x: Float) -> Float
```
Hyperbolic sine.

```ys
Function Cosh(x: Float) -> Float
```
Hyperbolic cosine.

```ys
Function Tanh(x: Float) -> Float
```
Hyperbolic tangent.

```ys
Function Sqrt(x: Float) -> Float
```
Square root.

```ys
Function Pow(base: Float, exp: Float) -> Float
```
`base` raised to `exp`.

```ys
Function Exp(x: Float) -> Float
```
`e^x`.

```ys
Function Log(x: Float) -> Float
```
Natural logarithm.

```ys
Function Log10(x: Float) -> Float
```
Base-10 logarithm.

```ys
Function Log2(x: Float) -> Float
```
Base-2 logarithm.

```ys
Function Ceil(x: Float) -> Float
```
Rounds up to nearest integer.

```ys
Function Floor(x: Float) -> Float
```
Rounds down to nearest integer.

```ys
Function Round(x: Float) -> Float
```
Rounds to nearest integer (half up).

```ys
Function Trunc(x: Float) -> Float
```
Truncates fractional part.

```ys
Function Random() -> Float
```
Returns a pseudo-random Float in `[0.0, 1.0)`.

```ys
Function RandomInt(min: Int, max: Int) -> Int
```
Returns a pseudo-random Int in `[min, max]` (inclusive).

```ys
Function RandomFloat(min: Float, max: Float) -> Float
```
Returns a pseudo-random Float in `[min, max)`.

```ys
Function SeedRandom(seed: Int) -> Void
```
Seeds the pseudo-random number generator.

```ys
Function PI() -> Float
```
Returns π (`3.141592653589793`).

```ys
Function E() -> Float
```
Returns e (`2.718281828459045`).

```ys
Function IsNaN(x: Float) -> Bool
```
Returns `true` if `x` is NaN.

```ys
Function IsInfinity(x: Float) -> Bool
```
Returns `true` if `x` is ±Inf.

```ys
Function DegreesToRadians(degrees: Float) -> Float
```
Converts degrees to radians.

```ys
Function RadiansToDegrees(radians: Float) -> Float
```
Converts radians to degrees.

---

## Memory

```ys
Function MemoryAddress(obj: ...) -> Int
```
Returns the memory address of a value (for debugging).

```ys
Function StackAlloc(size: Int) -> ...
```
Allocates `size` bytes on the stack (low-level).

```ys
Function CopyMem_(dst: ..., src: ..., size: Int) -> Void
```
Copies `size` bytes from `src` to `dst` (low-level).

```ys
Function SetMem_(ptr: ..., value: Int, size: Int) -> Void
```
Sets `size` bytes at `ptr` to `value` (low-level).

```ys
Function ZeroMem_(ptr: ..., size: Int) -> Void
```
Zeroes `size` bytes at `ptr`.

```ys
Function SizeOf(type: ...) -> Int
```
Returns the size of a type in bytes.

---

## Process

```ys
Function RunProcess(cmd: String, args: String[], wait: Bool) -> Int
```
Runs a process. If `wait` is true, returns exit code; otherwise returns PID.

```ys
Function KillProcess(pid: Int) -> Void
```
Kills the process with the given PID.

```ys
Function ProcessExists(pid: Int) -> Bool
```
Returns `true` if a process with the given PID exists.

```ys
Function WaitProcess(pid: Int) -> Int
```
Waits for a process to exit and returns its exit code.

```ys
Function RunBackground(cmd: String) -> Int
```
Runs a command in the background; returns PID.

```ys
Function IsProcessRunning(pid: Int) -> Bool
```
Returns `true` if the process is still running.

---

## Network

```ys
Function HttpGet(url: String) -> String
```
Performs an HTTP GET request and returns the response body.

```ys
Function HttpPost(url: String, data: String, contentType: String) -> String
```
Performs an HTTP POST request with `data` and returns the response body.

```ys
Function HttpPut(url: String, data: String, contentType: String) -> String
```
Performs an HTTP PUT request and returns the response body.

```ys
Function HttpDelete(url: String) -> String
```
Performs an HTTP DELETE request and returns the response body.

```ys
Function HttpHead(url: String) -> String
```
Returns response headers as a string.

```ys
Function DownloadFile(url: String, path: String) -> Void
```
Downloads a file from `url` and saves it to `path`.

```ys
Function PingHost(host: String) -> Bool
```
Returns `true` if `host` is reachable.

```ys
Function ResolveHost(host: String) -> String
```
Resolves a hostname to an IP address string.

```ys
Function EncodeUrl(url: String) -> String
```
URL-encodes a string.

```ys
Function DecodeUrl(url: String) -> String
```
URL-decodes a string.

```ys
Function EncodeBase64(data: String) -> String
```
Base64-encodes a string.

```ys
Function DecodeBase64(data: String) -> String
```
Base64-decodes a string.

---

## Type

```ys
Function TypeOf(value: ...) -> String
```
Returns the type name as a string: `"Int"`, `"Float"`, `"String"`, `"Bool"`, etc.

```ys
Function IsInt(value: ...) -> Bool
```
Returns `true` if the value is an Int.

```ys
Function IsFloat(value: ...) -> Bool
```
Returns `true` if the value is a Float.

```ys
Function IsString(value: ...) -> Bool
```
Returns `true` if the value is a String.

```ys
Function IsBool(value: ...) -> Bool
```
Returns `true` if the value is a Bool.

```ys
Function IsArray(value: ...) -> Bool
```
Returns `true` if the value is an array.

```ys
Function IsFunction(value: ...) -> Bool
```
Returns `true` if the value is a function reference.

```ys
Function IsNull(value: ...) -> Bool
```
Returns `true` if the value is null.

```ys
Function CanConvert(from: ..., to: String) -> Bool
```
Returns `true` if a value of type `from` can be converted to type named by `to`.
