# Chapter 10 · Aliases

[← Operators & Flow](09-operators.md) · [Next: Math Expressions →](11-math.md)

---

Aliases let you create shortcuts for commands you use frequently.

## Defining Aliases

```nes
alias <name> = <command>
```

```nes
alias g = git status
alias build = cargo build --release
alias home = cd $USERPROFILE
alias cls = clear
```

Once defined, using the alias name runs the full command:

```nes
g                              # runs: git status
build                          # runs: cargo build --release
home                           # runs: cd C:\Users\you
```

## Aliases with Arguments

Extra arguments are appended to the expanded command:

```nes
alias gr = grep
gr TODO src/main.rs            # runs: grep TODO src/main.rs
```

```nes
alias gc = git commit -m
gc "fix bug"                   # runs: git commit -m fix bug
```

## Listing Aliases

```nes
alias
```

With no arguments, prints all defined aliases:

```
g=git status
build=cargo build --release
home=cd C:\Users\you
```

## How Aliases Work Internally

Aliases are stored as shell variables with the prefix `_alias_`:

| Alias                                 | Internal Variable                      |
| ------------------------------------- | -------------------------------------- |
| `alias g = git status`                | `_alias_g = git status`                |
| `alias build = cargo build --release` | `_alias_build = cargo build --release` |

When Nes encounters an unknown command, it checks `_alias_<cmd>` before falling back to the system.

## Alias Lifetime

Aliases persist for the duration of the session (interactive mode) or script execution. They are not saved between sessions.

To make aliases permanent, put them in a startup script:

```nes
# aliases.nes
alias g = git status
alias ga = git add -A
alias gc = git commit -m
alias gp = git push
alias build = cargo build --release
alias run = cargo run
```

Then run it at the start of each session:

```nes
run aliases.nes
```

---

[← Operators & Flow](09-operators.md) · [Next: Math Expressions →](11-math.md)
