# Nes — The Nestea Shell & Language

A lightweight, fast custom shell **and** programming language for Windows, built from scratch in Rust.
Zero dependencies. Single binary. Two modes: **NesC** (shell) and **NesT** (language).

## Install

**Option 1 — One command** (PowerShell):

```powershell
git clone https://github.com/SandwichEater577/Nes.git; cd Nes; .\install.ps1
```

**Option 2 — Double-click:**

1. [Download the repo](https://github.com/SandwichEater577/Nes/archive/refs/heads/main.zip) and unzip
2. Double-click **install.bat**

**Option 3 — Manual:**

```powershell
cargo build --release --target x86_64-pc-windows-gnu
copy target\x86_64-pc-windows-gnu\release\nes.exe %USERPROFILE%\.nes\nes.exe
```

After install, open a **new terminal** and type `nes help`.

> **Requires:** [Rust](https://rustup.rs) + MinGW toolchain for building from source.

---

## NesC — The Shell

### Quick Start

```
nes                     show usage
nes enter-full          launch interactive shell
nes <command>           run a single command
nes run script.nes      run a .nes script
nes help                show command reference
```

### Features

- **30+ built-in commands** — files, navigation, text, system info
- **Control flow** — `if`/`else`/`end`, `for`/`end` with nesting
- **Variables** — `let name = world` → `echo hello $name`
- **User input** — `read name` prompts for input
- **Pipes** — `ls | grep src`
- **Redirects** — `echo hello > file.txt` and `>>`
- **Chaining** — `mkdir build && cd build`
- **Math** — `calc (2+3)*4^2`
- **Aliases** — `alias g = grep`
- **Scripts** — `run deploy.nes`
- **System fallback** — unknown commands run via `cmd.exe`

### Commands

| Category | Commands                                                             |
| -------- | -------------------------------------------------------------------- |
| Navigate | `cd` `ls` `ll` `pwd` `tree` `find` `which`                           |
| Files    | `cat` `head` `tail` `wc` `touch` `mkdir` `rm` `cp` `mv` `hex` `size` |
| Text     | `echo` `grep`                                                        |
| System   | `whoami` `hostname` `os` `env` `time` `date` `open` `clear`          |
| Shell    | `let` `set` `unset` `export` `alias` `history` `run` `read`          |
| Control  | `if`/`else`/`end` `for`/`end` `sleep` `exists` `count` `typeof`      |
| Math     | `calc <expr>`                                                        |
| Exit     | `exit` `quit`                                                        |

### Example Script (.nes)

```nes
# deploy.nes — build and verify
let name = myapp

if not exists src
  echo No src/ directory!
else
  echo Building $name...
  for f in files src
    echo  compiling $f
  end
  echo Done!
end
```

Run: `nes run deploy.nes`

---

## NesT — The Language

NesT is a typed programming language with real expressions, functions, and control flow.
Files use the `.nest` extension and are run with `nes run file.nest`.

### Types

| Type    | Example         |
| ------- | --------------- |
| `int`   | `42`, `-7`      |
| `float` | `3.14`, `0.5`   |
| `str`   | `"hello"`       |
| `bool`  | `true`, `false` |

### Syntax Overview

```nest
# Variables
let name = "Nes";
let version = 5;
let pi = 3.14;
let ready = true;

# Arithmetic
let result = (version * 2 + 10) % 7;

# Strings
println("Welcome to " + name + " v" + str(version));

# If / else
if result > 3 {
    println("big");
} else {
    println("small");
}

# For loop (range-based)
for i in 0..5 {
    println(i);
}

# While loop
let n = 10;
while n > 0 {
    n = n - 1;
}

# Functions
fn factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

println("5! = " + str(factorial(5)));
```

### Built-in Functions

| Function     | Description           |
| ------------ | --------------------- |
| `print(x)`   | Print without newline |
| `println(x)` | Print with newline    |
| `input()`    | Read line from user   |
| `len(s)`     | Length of string      |
| `type(x)`    | Type name as string   |
| `str(x)`     | Convert to string     |
| `int(x)`     | Convert to int        |
| `float(x)`   | Convert to float      |
| `abs(x)`     | Absolute value        |
| `sqrt(x)`    | Square root           |
| `min(a, b)`  | Minimum of two values |
| `max(a, b)`  | Maximum of two values |
| `pow(a, b)`  | Raise a to power b    |

### Operators

| Category   | Operators                   |
| ---------- | --------------------------- |
| Arithmetic | `+` `-` `*` `/` `%`         |
| Comparison | `==` `!=` `<` `>` `<=` `>=` |
| Logical    | `&&` `\|\|` `!`             |

### Example (.nest)

```nest
# fibonacci.nest
fn fib(n) {
    if n <= 1 { return n; }
    return fib(n - 1) + fib(n - 2);
}

for i in 0..15 {
    println("fib(" + str(i) + ") = " + str(fib(i)));
}
```

Run: `nes run fibonacci.nest`

---

## Learn

See the [Lessons](../Nes%20Lessons/) folder — 5 short lessons from basics to real scripts.
See the [Docs](../Nes%20Docs/) folder — complete reference manual.

## License

MIT
