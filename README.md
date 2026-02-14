# Nes — The Nestea Shell

A lightweight, fast custom shell for Windows built from scratch in Rust.
Zero dependencies. Single binary. 30+ built-in commands. Scripts with `if`/`for` loops.

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
cargo build --release
copy target\release\nes.exe C:\nes\nes.exe    # or anywhere on your PATH
```

After install, open a **new terminal** and type `nes help`.

> **Requires:** [Rust](https://rustup.rs) for building from source.
> If a pre-built `nes.exe` is in the repo, no build tools needed.

## Quick Start

```
nes                     show usage
nes enter-full          launch interactive shell
nes <command>           run a single command
nes run script.nes      run a .nes script
nes help                show command reference
```

## Features

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

## Commands

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

## Example Script

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

## Learn

See the [Lessons](../Nes%20Lessons/) folder — 5 short lessons from basics to real scripts.

## License

MIT
