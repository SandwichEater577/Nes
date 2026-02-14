# Chapter 7 · System

[← Text Processing](06-text-processing.md) · [Next: Variables →](08-variables.md)

---

Commands for system information, time, and environment.

## Identity

### `whoami` — Current User

```nes
whoami
```

Prints the current username (from `%USERNAME%` or `%USER%`).

```
SandwichEater
```

### `hostname` — Computer Name

```nes
hostname
```

Prints the machine name (from `%COMPUTERNAME%` or `%HOSTNAME%`).

```
DESKTOP-ABC123
```

### `os` — Operating System

```nes
os
```

Prints OS and CPU architecture.

```
windows/x86_64
```

---

## Time

### `time` — Current Date & Time

```nes
time
```

```
2026-02-14 15:30:42
```

Format: `YYYY-MM-DD HH:MM:SS` (24-hour, UTC+1)

### `date` — Current Date

```nes
date
```

```
2026-02-14
```

Format: `YYYY-MM-DD`

---

## Environment

### `env` — List All Variables

```nes
env
```

Prints every environment variable in `KEY=VALUE` format.

```
USERNAME=SandwichEater
COMPUTERNAME=DESKTOP-ABC123
PATH=C:\Windows\system32;C:\Windows;...
USERPROFILE=C:\Users\SandwichEater
TEMP=C:\Users\SandwichEater\AppData\Local\Temp
...
```

---

## Utilities

### `open` — Open with System Default

```nes
open <path>
```

Opens a file, folder, or URL with the default system application.

```nes
open .                          # open current folder in Explorer
open README.md                  # open in default text editor
open https://github.com         # open in default browser
open image.png                  # open in default image viewer
```

Uses `cmd /c start` internally.

### `clear` / `cls` — Clear Screen

```nes
clear
cls
```

Both work identically — clears the terminal screen using ANSI escape codes (`\x1b[2J\x1b[H`).

---

[← Text Processing](06-text-processing.md) · [Next: Variables →](08-variables.md)
