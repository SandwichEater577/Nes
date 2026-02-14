# Chapter 12 · Writing Scripts

[← Math Expressions](11-math.md) · [Next: System Fallback →](13-system-fallback.md)

---

Nes scripts are plain text files with the `.nes` extension. Each line is a command, executed top-to-bottom.

## Creating a Script

Create any text file ending in `.nes`:

```nes
echo === Setup ===
let name = myproject
mkdir $name
cd $name
touch README.md
echo # $name > README.md
echo Done!
```

Save as `setup.nes`.

## Running Scripts

### From the command line

```
nes run setup.nes
```

### From inside an interactive session

```nes
run setup.nes
```

### From another script

```nes
run lib.nes
run main.nes
```

Scripts can call other scripts.

---

## Script Rules

| Rule                   | Detail                                          |
| ---------------------- | ----------------------------------------------- |
| One command per line   | Each line is a separate command                 |
| Blank lines            | Skipped silently                                |
| Variables persist      | `let` on line 1 is available on line 50         |
| Aliases persist        | Defined aliases work for the rest of the script |
| `exit` stops execution | Script halts, remaining lines ignored           |
| All operators work     | `&&`, `\|`, `>`, `>>` function normally         |

---

## Patterns

### Project Scaffolding

```nes
let name = myapp
mkdir $name
cd $name
mkdir src
mkdir tests
mkdir docs
echo [package] > Cargo.toml
echo name = "$name" >> Cargo.toml
echo version = "0.1.0" >> Cargo.toml
echo fn main() {} > src/main.rs
echo Scaffolded $name!
```

### Build Automation

```nes
echo === Building ===
cargo build --release
echo === Build Complete ===
size target/release
time
```

### System Report

```nes
echo === System Report === > report.txt
whoami >> report.txt
hostname >> report.txt
os >> report.txt
date >> report.txt
time >> report.txt
echo Report saved!
cat report.txt
```

### Log Rotation

```nes
cp app.log app.log.bak
echo Log rotated > app.log
date >> app.log
echo Logs rotated!
```

### Batch Cleanup

```nes
echo Cleaning...
rm target
rm build
rm dist
rm node_modules
echo Clean!
```

### Environment Setup

```nes
export PROJECT_ROOT=C:\Projects\myapp
export BUILD_TYPE=release
cd $PROJECT_ROOT
alias build = cargo build --release
alias test = cargo test
echo Environment ready!
```

---

## Script Exit

`exit` or `quit` stops script execution immediately:

```nes
echo This runs
exit
echo This never runs
```

---

## Limitations

The current version of Nes scripting has these constraints:

| Feature                | Status                                        |
| ---------------------- | --------------------------------------------- |
| `if` / `else`          | Not yet supported                             |
| `for` / `while` loops  | Not yet supported                             |
| Functions / procedures | Not yet supported                             |
| Comments               | No syntax (use `echo` for notes)              |
| Error handling         | Commands print errors but execution continues |
| Variable types         | Strings only (use `calc` for math)            |
| Return values          | Not supported                                 |

---

[← Math Expressions](11-math.md) · [Next: System Fallback →](13-system-fallback.md)
