# Chapter 1 · Introduction

[← Table of Contents](../README.md) · [Next: Installation →](02-installation.md)

---

## What is Nes?

**Nes** (short for *Nestea*) is a custom command-line shell and scripting language for Windows, built from scratch in Rust. It combines the familiarity of Unix-style commands (`ls`, `cat`, `grep`, `tree`) with native Windows integration — all in a single, fast, portable binary.

Nes is both:
- **A shell** — an interactive command-line environment you type commands into
- **A language** — a scripting format (`.nes` files) for automating tasks

## Why Nes?

| Problem | Nes Solution |
|---------|------|
| `cmd.exe` syntax is arcane and painful | Clean, minimal syntax with `$variables` |
| PowerShell is verbose and slow to start | Instant startup, short commands |
| Unix commands don't exist on Windows | 30+ built-in Unix-style commands |
| `.bat` files are ugly | `.nes` scripts are readable |
| No built-in calculator | `calc` with full expression support |

## Features at a Glance

### Commands (30+)
Navigation, file management, text processing, system info — all built-in without external dependencies.

### Variable System
Shell variables (`let`), environment variables (`set`, `export`), and automatic `$expansion`.

### Pipes & Redirection
```nes
ls | grep .rs
echo hello > file.txt
cat log.txt >> archive.txt
```

### Command Chaining
```nes
mkdir build && cd build && echo ready
```

### Math Evaluator
```nes
calc (2+3)*4^2
```

### Script Execution
```nes
nes run deploy.nes
```

### System Fallback
Any command Nes doesn't know gets passed to `cmd.exe` — so `git`, `cargo`, `python`, and everything else just works.

## Design Philosophy

1. **Fast** — written in Rust with aggressive optimizations (LTO, single codegen unit, stripped symbols)
2. **Portable** — single `.exe`, no installer, no dependencies
3. **Familiar** — if you know Unix basics, you know Nes
4. **Practical** — solves real problems, doesn't try to be everything

---

[← Table of Contents](../README.md) · [Next: Installation →](02-installation.md)
