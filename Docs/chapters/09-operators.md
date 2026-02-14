# Chapter 9 · Operators & Flow

[← Variables](08-variables.md) · [Next: Aliases →](10-aliases.md)

---

Nes supports pipes, output redirection, command chaining, and quoting — the building blocks for composing complex workflows from simple commands.

## Command Chaining — `&&`

Run multiple commands in sequence on a single line:

```nes
mkdir build && cd build && echo ready
```

```nes
let name = app
mkdir $name && cd $name && touch main.rs && echo Project created
```

Each segment separated by `&&` is executed left-to-right.

---

## Pipes — `|`

Send the output of one command as input to the next:

```nes
ls | grep src
cat main.rs | grep fn
```

Pipes connect `stdout` → `stdin` using OS-level process pipes. Chains can be any length:

```nes
cat log.txt | grep ERROR | grep database
```

Both built-in and system commands participate in pipes:

```nes
git log --oneline | grep fix
cargo test 2>&1 | grep FAILED
```

---

## Output Redirection

### Write — `>`

Redirect output to a file, **overwriting** existing content:

```nes
echo Hello > greeting.txt
ls > filelist.txt
date > timestamp.txt
```

### Append — `>>`

Redirect output, **appending** to the end of the file:

```nes
echo First line > log.txt
echo Second line >> log.txt
echo Third line >> log.txt
```

Result in `log.txt`:

```
First line
Second line
Third line
```

### Redirecting any command

Both built-in and external commands can be redirected:

```nes
whoami > user.txt
git status > status.txt
tree > structure.txt
```

---

## Quoting

Use single `'` or double `"` quotes to group text containing spaces into a single argument:

```nes
echo "Hello World"
cd "My Documents"
grep "error code" log.txt
echo 'single quotes work too'
```

Quotes are stripped — the content becomes one argument. There is no difference between single and double quotes in Nes.

---

## Combining Everything

All operators work together:

```nes
mkdir output && ls src | grep .rs > output/rust_files.txt
```

```nes
let dir = build
mkdir $dir && echo Started > $dir/log.txt && date >> $dir/log.txt
```

```nes
echo === Report === > report.txt && whoami >> report.txt && date >> report.txt
```

---

## Execution Order

When Nes processes a line, it follows this sequence:

```
1. Variable expansion     $var → value
2. Split on &&            separate chain segments
3. For each segment:
   a. Pipes (|)           if found → pipe chain
   b. Append (>>)         if found → capture & append to file
   c. Write (>)           if found → capture & write to file
   d. Dispatch            execute as command
4. Command dispatch:
   a. Built-in?           → run built-in handler
   b. Alias?              → expand alias, re-execute
   c. Neither?            → pass to cmd /c
```

---

[← Variables](08-variables.md) · [Next: Aliases →](10-aliases.md)
