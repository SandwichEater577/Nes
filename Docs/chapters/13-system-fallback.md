# Chapter 13 · System Fallback

[← Writing Scripts](12-scripting.md) · [Next: Command Reference →](14-command-reference.md)

---

Any command that Nes doesn't recognize as a built-in is passed to the Windows system shell (`cmd /c`) for execution.

## How It Works

When you type a command, Nes checks in this order:

```
1. Is it a built-in command?     → run built-in
2. Is it an alias?               → expand and re-execute
3. Neither?                      → cmd /c <command>
```

This means **every program installed on your system** works inside Nes:

```nes
git status
git add -A
git commit -m "update"
git push

cargo build --release
cargo test
cargo run

python script.py
node server.js
npm install

ping google.com
ipconfig
tasklist
```

## Exit Codes

If an external command fails (non-zero exit code), Nes displays it in red:

```nes
cargo build
# exit 101
```

If the command isn't found at all:

```nes
nonexistent
# nes: 'nonexistent' not recognized
```

## Combining with Nes Features

External commands work with all Nes operators:

### Variables

```nes
let branch = main
git checkout $branch
```

### Pipes

```nes
git log --oneline | grep fix
```

### Redirection

```nes
git status > status.txt
cargo test 2>&1 >> build.log
```

### Chaining

```nes
git add -A && git commit -m "update" && git push
```

---

## .nes vs .bat

| Feature         | `.nes`                   | `.bat`              |
| --------------- | ------------------------ | ------------------- |
| Variable syntax | `$var`                   | `%var%`             |
| Set variable    | `let x = 5`              | `set x=5`           |
| Pipes           | `cmd1 \| cmd2`           | `cmd1 \| cmd2`      |
| Redirection     | `>` `>>`                 | `>` `>>`            |
| Math            | `calc expr`              | `set /a expr`       |
| Unix commands   | Built-in                 | Need external tools |
| File inspection | `wc` `hex` `size` `tree` | Complex workarounds |
| Color output    | ANSI built-in            | Limited             |
| Readability     | Clean, minimal           | Verbose             |
| Comments        | Not supported            | `REM` or `::`       |
| Conditionals    | Not yet                  | `if` `else`         |
| Loops           | Not yet                  | `for`               |

---

[← Writing Scripts](12-scripting.md) · [Next: Command Reference →](14-command-reference.md)
