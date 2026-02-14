# Chapter 6 · Text Processing

[← File Operations](05-file-operations.md) · [Next: System →](07-system.md)

---

Commands for writing and searching text.

## `echo` — Print Text

```nes
echo <text>
```

Prints text to the terminal (or to a file via redirection).

```nes
echo Hello, World!
echo Building project...
echo
```

### With variables

```nes
let name = Nes
echo Welcome to $name
# Welcome to Nes
```

### With redirection

```nes
echo # My Project > README.md           # write to file
echo Build log entry >> build.log        # append to file
```

### Multiple words

Everything after `echo` is treated as a single string — no quoting needed for spaces:

```nes
echo This is a full sentence with spaces
# This is a full sentence with spaces
```

---

## `grep` — Search in Files

```nes
grep <pattern> <file>
```

Searches a file line-by-line and prints lines that contain the pattern. Matches are highlighted in **red**.

```nes
grep fn src/main.rs
```

```
    fn new() -> Self {
    fn prompt(&self, out: &mut impl Write) {
    fn exec(&mut self, raw: &str, out: &mut BufWriter<io::StdoutLock<'_>>) {
    fn expand_vars(&self, input: &str) -> String {
    ...
```

### Examples

```nes
grep TODO src/main.rs          # find TODO comments
grep error log.txt             # search error logs
grep import script.py          # find Python imports
grep "fn main" src/main.rs     # find main function
```

### Behavior

- **Case-sensitive** — `grep Error` won't match `error`
- **Substring match** — `grep fn` matches `fn`, `function`, `define`
- **Highlighting** — matched text appears in red (ANSI colors)
- **Line-by-line** — each matching line is printed on its own line

### With pipes

```nes
cat src/main.rs | grep fn
ls | grep .rs
```

---

[← File Operations](05-file-operations.md) · [Next: System →](07-system.md)
