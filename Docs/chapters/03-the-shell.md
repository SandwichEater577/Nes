# Chapter 3 · The Shell

[← Installation](02-installation.md) · [Next: Navigation →](04-navigation.md)

---

Nes operates in three modes: **launcher**, **single command**, and **interactive shell**.

## Launcher Mode

Running `nes` with no arguments shows a usage summary:

```
$ nes
nes — the nestea shell v3.0
  nes <command>       run a single command
  nes enter-full      launch interactive shell
  nes run <file.nes>  run a script
  nes --completions   list all commands
  nes help            show all commands
```

## Single Command Mode

Pass a command directly:

```
$ nes ls
$ nes echo hello
$ nes calc 2+2
$ nes cat README.md
```

Nes executes the command and exits immediately. Useful for one-off operations or integrating Nes commands into other scripts.

Multiple arguments are joined and treated as a single command line:

```
$ nes echo hello world
hello world
```

## Interactive Mode

```
$ nes enter-full
```

Opens a persistent shell session:

```
nes — the nestea shell v3.0

C:\Users\you nes> _
```

### The Prompt

The prompt shows:

- **Current directory** in cyan
- **`nes>`** in yellow

```
C:\Users\you\Projects nes> ls
```

### Session Features

In interactive mode you get:

- **Command history** — all commands are recorded, view with `history`
- **Variables** — persist across commands in the same session
- **Aliases** — defined once, available for the rest of the session

### Exiting

```nes
exit
quit
```

Both print `Goodbye.` and close the session.

## Script Mode

```
$ nes run script.nes
```

Executes a `.nes` file line-by-line. See [Chapter 12 · Writing Scripts](12-scripting.md).

## Completions

```
$ nes --completions
```

Outputs a plain list of all built-in command names — one per line. Useful for integrating Nes with external autocomplete systems.

---

[← Installation](02-installation.md) · [Next: Navigation →](04-navigation.md)
