# Chapter 4 · Navigation

[← The Shell](03-the-shell.md) · [Next: File Operations →](05-file-operations.md)

---

Commands for moving through the filesystem and discovering what's where.

## `cd` — Change Directory

```nes
cd <path>
```

### Absolute path

```nes
cd C:\Users\you\Projects
```

### Relative path

```nes
cd src
cd ../tests
```

### Home directory

```nes
cd
```

With no arguments, jumps to `%USERPROFILE%` (typically `C:\Users\you`).

### Previous directory

```nes
cd -
```

Returns to the last directory you were in. Nes tracks this automatically in the `OLDPWD` variable.

### Errors

```nes
cd nonexistent
# cd: The system cannot find the path specified. (os error 3)
```

---

## `ls` — List Directory

```nes
ls [dir]
```

Lists contents of a directory. Directories appear in **blue** with a trailing `/`. Files appear uncolored. Everything is sorted alphabetically, directories first.

```nes
ls                    # current directory
ls src                # specific directory
ls C:\Windows         # absolute path
```

**Output format:** single line, space-separated.

```
 src/  target/  .gitignore  Cargo.lock  Cargo.toml  nes.exe
```

---

## `ll` — Long Listing

```nes
ll [dir]
```

Detailed listing with file sizes and directory markers.

```nes
ll
```

```
      <DIR>  src/
      <DIR>  target/
         8  .gitignore
       150  Cargo.lock
       434  Cargo.toml
    495616  nes.exe
```

Directories show `<DIR>` instead of a size. Files show byte count right-aligned.

---

## `pwd` — Print Working Directory

```nes
pwd
```

Prints the full absolute path of the current directory.

```
C:\Users\you\Projects\Nes
```

---

## `tree` — Directory Tree

```nes
tree [dir]
```

Displays a visual tree with box-drawing characters.

```nes
tree
tree src
tree C:\Projects
```

```
./
├── Cargo.toml
├── src/
│   └── main.rs
├── .gitignore
└── nes.exe
```

Recursively walks all subdirectories.

---

## `find` — Search Files

```nes
find [pattern]
```

Recursively searches the current directory for files whose names contain `pattern`.

```nes
find main          # files containing "main" in name
find .rs           # Rust source files
find config        # config files
find               # list everything (pattern = *)
```

Output is one file path per line:

```
.\src\main.rs
```

**Note:** This is a substring match, not a glob or regex.

---

## `which` — Locate Executable

```nes
which <name>
```

Searches the system `PATH` for an executable and prints its full path.

```nes
which git
# C:\Program Files\Git\cmd\git.exe

which cargo
# C:\Users\you\.cargo\bin\cargo.exe

which nonexistent
# which: 'nonexistent' not found
```

Looks for `<name>.exe` in each directory on `PATH`.

---

[← The Shell](03-the-shell.md) · [Next: File Operations →](05-file-operations.md)
