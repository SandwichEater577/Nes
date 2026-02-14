# Chapter 15 · Examples

[← Command Reference](14-command-reference.md) · [Back to Table of Contents](../README.md)

---

Complete, ready-to-run `.nes` script examples. Each can be saved as a `.nes` file and run with `nes run <filename>`.

---

## 1 · Hello World

```nes
echo === Hello from Nes! ===
let name = World
echo Greetings, $name
date
time
whoami
hostname
os
```

---

## 2 · System Report

```nes
echo === System Report ===
echo
echo User:
whoami
echo Host:
hostname
echo OS:
os
echo Time:
time
echo
echo === Current Directory ===
pwd
ls
echo
echo === Directory Size ===
size .
echo
echo === Directory Tree ===
tree .
```

---

## 3 · Project Scaffolder

```nes
let name = myproject
echo Creating $name...

mkdir $name
cd $name
mkdir src
mkdir tests
mkdir docs

echo # $name > README.md
echo >> README.md
echo A new project created with Nes. >> README.md

echo fn main() { > src/main.rs
echo     println!("Hello from $name!"); >> src/main.rs
echo } >> src/main.rs

touch tests/test.rs
touch docs/guide.md

echo
echo Project $name created!
tree .
```

---

## 4 · Math Showcase

```nes
echo === Nes Calculator ===
echo
echo Basic:
calc 2+2
calc 100-37
calc 6*7
calc 144/12
echo
echo Powers:
calc 2^8
calc 2^16
calc 2^32
echo
echo Precedence:
calc 2+3*4
calc (2+3)*4
echo
echo Conversions:
echo Bytes in 1 GB:
calc 1024*1024*1024
echo Seconds in a day:
calc 24*60*60
echo 100C to Fahrenheit:
calc 9/5*100+32
```

---

## 5 · File Operations Demo

```nes
echo === File Ops Demo ===

mkdir demo_workspace
cd demo_workspace

echo Hello World > hello.txt
echo Line 1 > multi.txt
echo Line 2 >> multi.txt
echo Line 3 >> multi.txt
echo Line 4 >> multi.txt
echo Line 5 >> multi.txt

echo --- cat ---
cat hello.txt

echo --- head 3 ---
head 3 multi.txt

echo --- tail 2 ---
tail 2 multi.txt

echo --- wc ---
wc multi.txt

echo --- grep ---
grep Line multi.txt

echo --- directory ---
ls
size .

echo Cleaning up...
cd ..
rm demo_workspace
echo Done!
```

---

## 6 · Git Workflow

```nes
echo === Git Quick Push ===
git add -A
git status
git commit -m "update"
git push
echo Pushed!
date
```

---

## 7 · Development Aliases

```nes
echo Setting up dev aliases...
alias g = git status
alias ga = git add -A
alias gc = git commit -m
alias gp = git push
alias build = cargo build --release
alias run = cargo run
alias test = cargo test
echo Aliases ready!
alias
```

---

## 8 · Report Generator

```nes
let report = report.txt
echo === Report === > $report
echo Generated: >> $report
date >> $report
time >> $report
echo >> $report
echo User: >> $report
whoami >> $report
echo Host: >> $report
hostname >> $report
echo OS: >> $report
os >> $report
echo >> $report
echo Directory: >> $report
pwd >> $report
echo >> $report
echo Files: >> $report
ls >> $report
echo
echo Report saved to $report
cat $report
```

---

## 9 · Multi-Project Build

```nes
echo === Building all projects ===

cd project-a
echo Building project-a...
cargo build --release
echo Done!
cd ..

cd project-b
echo Building project-b...
cargo build --release
echo Done!
cd ..

echo === All builds complete ===
date
```

---

## 10 · Variable Showcase

```nes
echo === Variables ===

let greeting = Hello
let target = World
echo $greeting, $target!

let version = 3.0
echo Nes v$version

echo
echo Environment:
echo User: $USERNAME
echo Home: $USERPROFILE
echo Temp: $TEMP

echo
echo Setting custom env:
export MY_APP=running
echo MY_APP = $MY_APP
```

---

[← Command Reference](14-command-reference.md) · [Back to Table of Contents](../README.md)
