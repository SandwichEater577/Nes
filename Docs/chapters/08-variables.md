# Chapter 8 · Variables

[← System](07-system.md) · [Next: Operators & Flow →](09-operators.md)

---

Nes has a variable system with shell-local variables, environment variables, and automatic expansion.

## Defining Variables

### `let` — Shell Variable

```nes
let name = value
```

Creates a variable that exists only in the current Nes session.

```nes
let project = myapp
let version = 1.0.0
let greeting = Hello World
```

### `set` — Environment Variable

```nes
set KEY=value
```

Sets a process-level environment variable (visible to child processes).

```nes
set MY_VAR=hello
set BUILD_TYPE=release
```

With no arguments, `set` lists all shell variables:

```nes
set
# project=myapp
# version=1.0.0
```

### `export` — Both at Once

```nes
export KEY=value
```

Sets both a shell variable and an environment variable simultaneously.

```nes
export PROJECT=myapp
```

Now `$PROJECT` works in Nes and child processes (like `cargo`, `git`) can see it too.

---

## Using Variables

Prefix a variable name with `$` to expand its value:

```nes
let name = Nes
echo Hello from $name          # Hello from Nes
```

### Lookup Order

1. Shell variables (`let`) are checked first
2. Environment variables are checked second
3. If not found, the literal `$name` text is kept

```nes
let x = hello
echo $x                        # "hello"        — shell var
echo $USERNAME                 # "YourName"     — env var
echo $nonexistent              # "$nonexistent" — not found
```

### Valid Variable Names

Variable names can contain:

- Letters: `a-z`, `A-Z`
- Digits: `0-9`
- Underscores: `_`

The name ends at the first character that isn't one of these:

```nes
let my_var = test
echo $my_var!              # "test!"     — ! is not part of the name
echo $my_var.txt           # "test.txt"  — . ends the name
echo $my_var/path          # "test/path" — / ends the name
```

---

## Expansion Context

Variables expand **everywhere** — in arguments, paths, redirections:

```nes
let dir = src
ls $dir                        # ls src

let file = output.txt
echo hello > $file             # write to output.txt
echo more >> $file             # append to output.txt

let name = myproject
mkdir $name && cd $name        # create and enter directory
```

---

## Removing Variables

### `unset` — Remove Shell Variable

```nes
unset project
```

Removes the variable from the shell. Does not affect environment variables.

---

## Listing Variables

```nes
set                            # all shell variables with values
env                            # all environment variables
```

---

## Built-in Variables

| Variable        | Set By  | Description                |
| --------------- | ------- | -------------------------- |
| `OLDPWD`        | `cd`    | Previous working directory |
| `_alias_<name>` | `alias` | Internal alias storage     |

## Common Environment Variables

All standard Windows environment variables are accessible:

```nes
echo $USERNAME                 # current user
echo $COMPUTERNAME             # machine name
echo $USERPROFILE              # home directory path
echo $PATH                     # executable search path
echo $TEMP                     # temp directory
echo $APPDATA                  # application data
echo $PROGRAMFILES             # Program Files directory
```

---

[← System](07-system.md) · [Next: Operators & Flow →](09-operators.md)
