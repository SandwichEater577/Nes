# Chapter 2 · Installation

[← Introduction](01-introduction.md) · [Next: The Shell →](03-the-shell.md)

---

## Requirements

- **OS:** Windows 10/11
- **Build tools:** Rust toolchain (`rustup`, `cargo`)

## Building from Source

### 1. Clone the repository

```bash
git clone https://github.com/SandwichEater577/Nes.git
cd Nes
```

### 2. Build in release mode

```bash
cargo build --release
```

The binary is compiled with maximum optimizations:

| Setting         | Value   | Effect                                          |
| --------------- | ------- | ----------------------------------------------- |
| `opt-level`     | `3`     | Maximum optimization                            |
| `lto`           | `true`  | Link-Time Optimization (whole-program inlining) |
| `codegen-units` | `1`     | Single codegen unit for best optimization       |
| `panic`         | `abort` | No unwinding overhead                           |
| `strip`         | `true`  | Debug symbols removed                           |

### 3. Locate the binary

```
target/release/nes.exe
```

### 4. Add to PATH

Copy `nes.exe` somewhere on your system `PATH`, or add its directory:

```bash
set PATH=C:\path\to\nes;%PATH%
```

## Using the Pre-built Binary

If `nes.exe` is included in the repository, you can use it directly:

```bash
.\nes.exe help
```

## Verify Installation

```bash
nes help
```

You should see the command reference listing all categories.

---

[← Introduction](01-introduction.md) · [Next: The Shell →](03-the-shell.md)
