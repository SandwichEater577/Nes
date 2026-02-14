# Chapter 5 · File Operations

[← Navigation](04-navigation.md) · [Next: Text Processing →](06-text-processing.md)

---

Commands for reading, creating, copying, moving, and inspecting files.

## Reading Files

### `cat` — Display File Contents

```nes
cat <file>
```

Prints the entire file to the terminal.

```nes
cat README.md
cat src/main.rs
```

If the file doesn't end with a newline, one is added automatically.

### `head` — First N Lines

```nes
head [n] <file>
```

Shows the first N lines (default: 10).

```nes
head src/main.rs          # first 10 lines
head 5 src/main.rs        # first 5 lines
head 1 Cargo.toml         # first line only
```

### `tail` — Last N Lines

```nes
tail [n] <file>
```

Shows the last N lines (default: 10).

```nes
tail src/main.rs          # last 10 lines
tail 20 src/main.rs       # last 20 lines
tail 3 log.txt            # last 3 lines
```

### `wc` — Word Count

```nes
wc <file>
```

Reports lines, words, and bytes.

```nes
wc src/main.rs
```

```
  685L  2341W  28725B  src/main.rs
```

| Field | Meaning                           |
| ----- | --------------------------------- |
| `L`   | Line count                        |
| `W`   | Word count (whitespace-delimited) |
| `B`   | Byte count                        |

---

## Creating Files

### `touch` — Create Empty File

```nes
touch <file>
```

Creates a new empty file. If the file already exists, it's not modified.

```nes
touch notes.txt
touch src/utils.rs
```

### `mkdir` — Create Directory

```nes
mkdir <path>
```

Creates a directory and all necessary parent directories.

```nes
mkdir src
mkdir src/utils/helpers     # creates src/, src/utils/, and src/utils/helpers/
mkdir build/release
```

---

## Moving & Copying

### `cp` — Copy File

```nes
cp <source> <destination>
```

```nes
cp config.toml config.backup.toml
cp src/main.rs src/main.rs.bak
```

### `mv` — Move / Rename

```nes
mv <source> <destination>
```

```nes
mv old_name.rs new_name.rs          # rename
mv temp.txt archive/temp.txt        # move to directory
```

---

## Deleting

### `rm` — Remove File or Directory

```nes
rm <path>
```

If the path is a file, deletes the file. If it's a directory, deletes it **recursively** (all contents included).

```nes
rm temp.txt              # delete file
rm build                 # delete entire directory
rm target/debug          # delete subdirectory
```

> **Warning:** Deletion is permanent. Files do not go to the Recycle Bin.

---

## Inspection

### `hex` — Hex Dump

```nes
hex <file>
```

Displays the first 512 bytes of a file in hexadecimal + ASCII format.

```nes
hex nes.exe
```

```
00000000  4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00  |MZ..............|
00000010  b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00  |........@.......|
...
... (495616 bytes total)
```

Each line shows:

- **Offset** (hex)
- **16 bytes** in hex
- **ASCII representation** (non-printable chars shown as `.`)

For files longer than 512 bytes, shows `... (N bytes total)`.

### `size` — File/Directory Size

```nes
size <path>
```

Calculates total size with human-readable units.

```nes
size nes.exe             # 483.9 KB
size .                   # 12.3 MB
size target              # 45.7 MB
size src/main.rs         # 28.1 KB
```

For directories, recursively sums all file sizes.

| Unit | Threshold             |
| ---- | --------------------- |
| `B`  | < 1,024 bytes         |
| `KB` | < 1,048,576 bytes     |
| `MB` | < 1,073,741,824 bytes |
| `GB` | < 1 TB                |
| `TB` | ≥ 1 TB                |

---

[← Navigation](04-navigation.md) · [Next: Text Processing →](06-text-processing.md)
