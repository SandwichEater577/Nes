# Chapter 14 · Command Reference

[← System Fallback](13-system-fallback.md) · [Next: Examples →](15-examples.md)

---

Quick-lookup table of every built-in command.

## Navigation — 7 commands

| Command | Syntax           | Description                                     |
| ------- | ---------------- | ----------------------------------------------- |
| `cd`    | `cd [dir]`       | Change directory. No arg = home. `-` = previous |
| `ls`    | `ls [dir]`       | List directory (colored, sorted)                |
| `ll`    | `ll [dir]`       | Long listing with sizes                         |
| `pwd`   | `pwd`            | Print working directory                         |
| `tree`  | `tree [dir]`     | Visual directory tree                           |
| `find`  | `find [pattern]` | Recursive file search (substring)               |
| `which` | `which <name>`   | Locate executable on PATH                       |

## File Operations — 11 commands

| Command | Syntax            | Description                  |
| ------- | ----------------- | ---------------------------- |
| `cat`   | `cat <file>`      | Display file contents        |
| `head`  | `head [n] <file>` | First N lines (default 10)   |
| `tail`  | `tail [n] <file>` | Last N lines (default 10)    |
| `wc`    | `wc <file>`       | Count lines, words, bytes    |
| `touch` | `touch <file>`    | Create empty file            |
| `mkdir` | `mkdir <path>`    | Create directory (recursive) |
| `rm`    | `rm <path>`       | Delete file or directory     |
| `cp`    | `cp <src> <dst>`  | Copy file                    |
| `mv`    | `mv <src> <dst>`  | Move / rename                |
| `hex`   | `hex <file>`      | Hex dump (first 512 bytes)   |
| `size`  | `size <path>`     | Human-readable size          |

## Text — 2 commands

| Command | Syntax                  | Description                    |
| ------- | ----------------------- | ------------------------------ |
| `echo`  | `echo <text>`           | Print text                     |
| `grep`  | `grep <pattern> <file>` | Search file, highlight matches |

## System — 8 commands

| Command    | Syntax          | Description                |
| ---------- | --------------- | -------------------------- |
| `whoami`   | `whoami`        | Current username           |
| `hostname` | `hostname`      | Computer name              |
| `os`       | `os`            | OS and architecture        |
| `env`      | `env`           | List environment variables |
| `time`     | `time`          | Current date & time        |
| `date`     | `date`          | Current date               |
| `open`     | `open <path>`   | Open with system default   |
| `clear`    | `clear` / `cls` | Clear screen               |

## Shell — 7 commands

| Command   | Syntax               | Description              |
| --------- | -------------------- | ------------------------ |
| `let`     | `let name = value`   | Set shell variable       |
| `set`     | `set [key=val]`      | List or set env variable |
| `unset`   | `unset <name>`       | Remove shell variable    |
| `export`  | `export key=val`     | Set shell + env variable |
| `alias`   | `alias [name = cmd]` | Define or list aliases   |
| `history` | `history`            | Show command history     |
| `run`     | `run <file.nes>`     | Execute a script         |

## Math — 1 command

| Command | Syntax        | Description              |
| ------- | ------------- | ------------------------ |
| `calc`  | `calc <expr>` | Evaluate math expression |

## Exit — 2 commands

| Command | Syntax | Description    |
| ------- | ------ | -------------- |
| `exit`  | `exit` | Exit the shell |
| `quit`  | `quit` | Exit the shell |

---

## Operators

| Operator | Syntax               | Description                 |
| -------- | -------------------- | --------------------------- |
| Chain    | `cmd1 && cmd2`       | Run commands in sequence    |
| Pipe     | `cmd1 \| cmd2`       | Connect stdout → stdin      |
| Write    | `cmd > file`         | Redirect output (overwrite) |
| Append   | `cmd >> file`        | Redirect output (append)    |
| Quote    | `"text"` or `'text'` | Group words as one argument |

---

**Total: 38 built-in commands + 5 operators**

---

[← System Fallback](13-system-fallback.md) · [Next: Examples →](15-examples.md)
