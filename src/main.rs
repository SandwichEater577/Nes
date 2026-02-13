use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufWriter, Write};
use std::process::{Command, Stdio};
use std::time::SystemTime;

struct Shell {
    vars: HashMap<String, String>,
    history: Vec<String>,
    running: bool,
}

impl Shell {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            history: Vec::with_capacity(512),
            running: true,
        }
    }

    fn prompt(&self, out: &mut impl Write) {
        let cwd = env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "?".into());
        let _ = write!(out, "\x1b[36m{}\x1b[0m \x1b[33mnes>\x1b[0m ", cwd);
        let _ = out.flush();
    }

    fn exec(&mut self, raw: &str, out: &mut BufWriter<io::StdoutLock<'_>>) {
        let raw = self.expand_vars(raw);
        for chain in raw.split("&&") {
            let chain = chain.trim();
            if chain.is_empty() { continue; }
            if chain.contains('|') {
                self.exec_pipe(chain, out);
            } else if let Some(pos) = chain.find(">>") {
                let (cmd, file) = (&chain[..pos], chain[pos + 2..].trim());
                let capture = self.capture(cmd.trim());
                let _ = fs::OpenOptions::new().create(true).append(true).open(file)
                    .and_then(|mut f| f.write_all(capture.as_bytes()));
            } else if let Some(pos) = chain.find('>') {
                let (cmd, file) = (&chain[..pos], chain[pos + 1..].trim());
                let capture = self.capture(cmd.trim());
                let _ = fs::write(file, capture);
            } else {
                self.dispatch(chain, out);
                let _ = out.flush();
            }
        }
    }

    fn expand_vars(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '$' && i + 1 < chars.len() && chars[i + 1].is_alphanumeric() {
                i += 1;
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') { i += 1; }
                let name: String = chars[start..i].iter().collect();
                if let Some(val) = self.vars.get(&name) {
                    result.push_str(val);
                } else if let Ok(val) = env::var(&name) {
                    result.push_str(&val);
                } else {
                    result.push('$');
                    result.push_str(&name);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    fn capture(&mut self, input: &str) -> String {
        let parts = Self::split_args(input);
        if parts.is_empty() { return String::new(); }
        match Command::new(&parts[0]).args(&parts[1..]).output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).into_owned(),
            Err(e) => format!("Error: {}\n", e),
        }
    }

    fn exec_pipe(&mut self, chain: &str, out: &mut BufWriter<io::StdoutLock<'_>>) {
        let cmds: Vec<&str> = chain.split('|').collect();
        if cmds.len() < 2 { self.dispatch(chain, out); return; }
        let mut prev_stdout: Option<Stdio> = None;
        let mut children = Vec::new();
        for (i, cmd) in cmds.iter().enumerate() {
            let parts = Self::split_args(cmd.trim());
            if parts.is_empty() { continue; }
            let stdin = prev_stdout.take().unwrap_or(Stdio::inherit());
            let stdout = if i < cmds.len() - 1 { Stdio::piped() } else { Stdio::inherit() };
            match Command::new(&parts[0]).args(&parts[1..]).stdin(stdin).stdout(stdout).spawn() {
                Ok(child) => {
                    if i < cmds.len() - 1 {
                        if child.stdout.is_some() {
                            prev_stdout = None;
                        }
                    }
                    children.push(child);
                    if let Some(ref child_out) = children.last().and_then(|c| c.stdout.as_ref()) {
                        use std::os::windows::io::{AsRawHandle, FromRawHandle};
                        let handle = child_out.as_raw_handle();
                        prev_stdout = Some(unsafe { Stdio::from_raw_handle(handle) });
                    }
                }
                Err(e) => { let _ = write!(out, "Error: {}\n", e); return; }
            }
        }
        for mut c in children { let _ = c.wait(); }
    }

    fn dispatch(&mut self, input: &str, out: &mut BufWriter<io::StdoutLock<'_>>) {
        let parts = Self::split_args(input);
        if parts.is_empty() { return; }
        let cmd = parts[0].as_str();
        let args = &parts[1..];
        let arg_str = if args.is_empty() { String::new() } else { args.join(" ") };
        match cmd {
            "exit" | "quit" => {
                let _ = out.write_all(b"\x1b[33mGoodbye.\x1b[0m\n");
                self.running = false;
            }
            "help" => self.write_help(out),
            "cd" => {
                let dir = if arg_str.is_empty() {
                    env::var("USERPROFILE").unwrap_or_else(|_| ".".into())
                } else if arg_str == "-" {
                    self.vars.get("OLDPWD").cloned().unwrap_or_else(|| ".".into())
                } else {
                    arg_str.clone()
                };
                let old = env::current_dir().ok().map(|p| p.to_string_lossy().into_owned());
                if let Err(e) = env::set_current_dir(&dir) {
                    let _ = write!(out, "cd: {}\n", e);
                } else if let Some(old) = old {
                    self.vars.insert("OLDPWD".into(), old);
                }
            }
            "let" => {
                if let Some(eq) = arg_str.find('=') {
                    let name = arg_str[..eq].trim().to_string();
                    let val = arg_str[eq + 1..].trim().to_string();
                    self.vars.insert(name, val);
                } else {
                    let _ = out.write_all(b"Usage: let name = value\n");
                }
            }
            "echo" => { let _ = write!(out, "{}\n", arg_str); }
            "set" => {
                if arg_str.is_empty() {
                    for (k, v) in &self.vars {
                        let _ = write!(out, "{}={}\n", k, v);
                    }
                } else if let Some(eq) = arg_str.find('=') {
                    let k = arg_str[..eq].trim();
                    let v = arg_str[eq + 1..].trim();
                    unsafe { env::set_var(k, v); }
                }
            }
            "unset" => { self.vars.remove(&arg_str as &str); }
            "export" => {
                if let Some(eq) = arg_str.find('=') {
                    let k = arg_str[..eq].trim();
                    let v = arg_str[eq + 1..].trim();
                    unsafe { env::set_var(k, v); }
                    self.vars.insert(k.to_string(), v.to_string());
                }
            }
            "history" => {
                for (i, h) in self.history.iter().enumerate() {
                    let _ = write!(out, "  {} {}\n", i + 1, h);
                }
            }
            "pwd" => {
                if let Ok(d) = env::current_dir() {
                    let _ = write!(out, "{}\n", d.display());
                }
            }
            "ls" => {
                let dir = if arg_str.is_empty() { ".".into() } else { arg_str };
                match fs::read_dir(&dir) {
                    Ok(entries) => {
                        let mut dirs = Vec::new();
                        let mut files = Vec::new();
                        for e in entries.flatten() {
                            let n = e.file_name().to_string_lossy().into_owned();
                            if e.path().is_dir() { dirs.push(n); } else { files.push(n); }
                        }
                        dirs.sort_unstable();
                        files.sort_unstable();
                        for d in &dirs { let _ = write!(out, " \x1b[34m{}/\x1b[0m", d); }
                        for f in &files { let _ = write!(out, " {}", f); }
                        if !dirs.is_empty() || !files.is_empty() { let _ = out.write_all(b"\n"); }
                    }
                    Err(e) => { let _ = write!(out, "ls: {}\n", e); }
                }
            }
            "ll" => {
                let dir = if arg_str.is_empty() { ".".into() } else { arg_str };
                if let Ok(entries) = fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        let meta = e.metadata();
                        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                        let name = e.file_name().to_string_lossy().into_owned();
                        if e.path().is_dir() {
                            let _ = write!(out, " \x1b[34m{:>10}  {}/\x1b[0m\n", "<DIR>", name);
                        } else {
                            let _ = write!(out, "  {:>10}  {}\n", size, name);
                        }
                    }
                }
            }
            "cat" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: cat <file>\n"); return; }
                match fs::read(&arg_str) {
                    Ok(data) => {
                        let _ = out.write_all(&data);
                        if data.last() != Some(&b'\n') { let _ = out.write_all(b"\n"); }
                    }
                    Err(e) => { let _ = write!(out, "cat: {}\n", e); }
                }
            }
            "head" => {
                let (n, file) = Self::parse_num_arg(args, 10);
                if let Ok(content) = fs::read_to_string(&file) {
                    for line in content.lines().take(n) {
                        let _ = write!(out, "{}\n", line);
                    }
                }
            }
            "tail" => {
                let (n, file) = Self::parse_num_arg(args, 10);
                if let Ok(content) = fs::read_to_string(&file) {
                    let lines: Vec<&str> = content.lines().collect();
                    let start = lines.len().saturating_sub(n);
                    for line in &lines[start..] {
                        let _ = write!(out, "{}\n", line);
                    }
                }
            }
            "wc" => {
                if let Ok(content) = fs::read_to_string(&arg_str) {
                    let lines = content.lines().count();
                    let words = content.split_whitespace().count();
                    let bytes = content.len();
                    let _ = write!(out, "  {}L  {}W  {}B  {}\n", lines, words, bytes, arg_str);
                }
            }
            "touch" => { let _ = fs::OpenOptions::new().create(true).append(true).open(&arg_str); }
            "mkdir" => { let _ = fs::create_dir_all(&arg_str); }
            "rm" => {
                let path = std::path::Path::new(arg_str.as_str());
                if path.is_dir() { let _ = fs::remove_dir_all(path); }
                else { let _ = fs::remove_file(path); }
            }
            "cp" => {
                if args.len() >= 2 { let _ = fs::copy(&args[0], &args[1]); }
                else { let _ = out.write_all(b"Usage: cp <src> <dst>\n"); }
            }
            "mv" => {
                if args.len() >= 2 { let _ = fs::rename(&args[0], &args[1]); }
                else { let _ = out.write_all(b"Usage: mv <src> <dst>\n"); }
            }
            "grep" => {
                if args.len() < 2 { let _ = out.write_all(b"Usage: grep <pattern> <file>\n"); return; }
                if let Ok(content) = fs::read_to_string(&args[1]) {
                    for line in content.lines() {
                        if line.contains(args[0].as_str()) {
                            let highlighted = line.replace(args[0].as_str(),
                                &format!("\x1b[31m{}\x1b[0m", args[0]));
                            let _ = write!(out, "{}\n", highlighted);
                        }
                    }
                }
            }
            "find" => {
                let pattern = if arg_str.is_empty() { "*" } else { &arg_str };
                Self::find_recursive(std::path::Path::new("."), pattern, out);
            }
            "tree" => {
                let dir = if arg_str.is_empty() { "." } else { &arg_str };
                Self::print_tree(std::path::Path::new(dir), "", true, out);
            }
            "whoami" => {
                let u = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or("unknown".into());
                let _ = write!(out, "{}\n", u);
            }
            "hostname" => {
                let h = env::var("COMPUTERNAME").or_else(|_| env::var("HOSTNAME")).unwrap_or("unknown".into());
                let _ = write!(out, "{}\n", h);
            }
            "os" => { let _ = write!(out, "{}/{}\n", env::consts::OS, env::consts::ARCH); }
            "env" => {
                for (k, v) in env::vars() {
                    let _ = write!(out, "{}={}\n", k, v);
                }
            }
            "time" => {
                let (y, mo, d, h, mi, s) = unix_to_datetime(unix_secs());
                let _ = write!(out, "{:04}-{:02}-{:02} {:02}:{:02}:{:02}\n", y, mo, d, h, mi, s);
            }
            "date" => {
                let (y, mo, d, _, _, _) = unix_to_datetime(unix_secs());
                let _ = write!(out, "{:04}-{:02}-{:02}\n", y, mo, d);
            }
            "calc" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: calc <expr>\n"); return; }
                let expr: String = arg_str.chars().filter(|c| *c != ' ').collect();
                match eval_expr(&expr) {
                    Ok(r) if r == r.floor() && r.abs() < 1e15 => { let _ = write!(out, "{}\n", r as i64); }
                    Ok(r) => { let _ = write!(out, "{}\n", r); }
                    Err(e) => { let _ = write!(out, "calc: {}\n", e); }
                }
            }
            "open" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: open <path>\n"); return; }
                let _ = out.flush();
                let _ = Command::new("cmd").args(["/c", "start", &arg_str]).spawn();
            }
            "clear" | "cls" => {
                let _ = out.write_all(b"\x1b[2J\x1b[H");
            }
            "run" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: run <script.nes>\n"); return; }
                if let Ok(script) = fs::read_to_string(&arg_str) {
                    for line in script.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        self.exec(line, out);
                        if !self.running { break; }
                    }
                } else {
                    let _ = write!(out, "run: cannot read '{}'\n", arg_str);
                }
            }
            "which" => {
                if let Ok(path) = env::var("PATH") {
                    for dir in path.split(';') {
                        let p = std::path::Path::new(dir).join(format!("{}.exe", arg_str));
                        if p.exists() { let _ = write!(out, "{}\n", p.display()); return; }
                    }
                }
                let _ = write!(out, "which: '{}' not found\n", arg_str);
            }
            "alias" => {
                if let Some(eq) = arg_str.find('=') {
                    let name = arg_str[..eq].trim().to_string();
                    let val = arg_str[eq + 1..].trim().to_string();
                    self.vars.insert(format!("_alias_{}", name), val);
                } else {
                    for (k, v) in &self.vars {
                        if let Some(name) = k.strip_prefix("_alias_") {
                            let _ = write!(out, "{}={}\n", name, v);
                        }
                    }
                }
            }
            "size" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: size <path>\n"); return; }
                let path = std::path::Path::new(arg_str.as_str());
                let total = Self::dir_size(path);
                let _ = write!(out, "{}\n", Self::human_size(total));
            }
            "hex" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: hex <file>\n"); return; }
                if let Ok(data) = fs::read(&arg_str) {
                    for (i, chunk) in data.chunks(16).enumerate().take(32) {
                        let _ = write!(out, "{:08x}  ", i * 16);
                        for b in chunk { let _ = write!(out, "{:02x} ", b); }
                        for _ in 0..(16 - chunk.len()) { let _ = out.write_all(b"   "); }
                        let _ = out.write_all(b" |");
                        for &b in chunk {
                            let c = if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' };
                            let _ = write!(out, "{}", c);
                        }
                        let _ = out.write_all(b"|\n");
                    }
                    if data.len() > 512 {
                        let _ = write!(out, "... ({} bytes total)\n", data.len());
                    }
                }
            }
            _ => {
                let alias_key = format!("_alias_{}", cmd);
                if let Some(expansion) = self.vars.get(&alias_key).cloned() {
                    let full = if arg_str.is_empty() { expansion } else { format!("{} {}", expansion, arg_str) };
                    self.exec(&full, out);
                    return;
                }
                let _ = out.flush();
                let status = Command::new("cmd")
                    .args(["/c", input])
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status();
                match status {
                    Ok(s) if !s.success() => {
                        let _ = write!(out, "\x1b[31mexit {}\x1b[0m\n", s.code().unwrap_or(-1));
                    }
                    Err(_) => { let _ = write!(out, "nes: '{}' not recognized\n", cmd); }
                    _ => {}
                }
            }
        }
    }

    fn split_args(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        let mut quote_char = ' ';
        for c in input.chars() {
            if in_quote {
                if c == quote_char { in_quote = false; }
                else { current.push(c); }
            } else if c == '"' || c == '\'' {
                in_quote = true;
                quote_char = c;
            } else if c == ' ' {
                if !current.is_empty() { args.push(std::mem::take(&mut current)); }
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() { args.push(current); }
        args
    }

    fn parse_num_arg(args: &[String], default: usize) -> (usize, String) {
        if args.len() >= 2 {
            let n = args[0].parse().unwrap_or(default);
            (n, args[1].clone())
        } else if args.len() == 1 {
            (default, args[0].clone())
        } else {
            (default, String::new())
        }
    }

    fn find_recursive(dir: &std::path::Path, pattern: &str, out: &mut impl Write) {
        if let Ok(entries) = fs::read_dir(dir) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                let path = e.path();
                if name.contains(pattern) || pattern == "*" {
                    let _ = write!(out, "{}\n", path.display());
                }
                if path.is_dir() { Self::find_recursive(&path, pattern, out); }
            }
        }
    }

    fn print_tree(dir: &std::path::Path, prefix: &str, is_last: bool, out: &mut impl Write) {
        let name = dir.file_name().map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.to_string_lossy().into_owned());
        let connector = if prefix.is_empty() { "" } else if is_last { "└── " } else { "├── " };
        if dir.is_dir() {
            let _ = write!(out, "{}{}\x1b[34m{}/\x1b[0m\n", prefix, connector, name);
        } else {
            let _ = write!(out, "{}{}{}\n", prefix, connector, name);
        }
        if dir.is_dir() {
            if let Ok(mut entries) = fs::read_dir(dir) {
                let items: Vec<_> = entries.by_ref().flatten().collect();
                let child_prefix = format!("{}{}", prefix,
                    if prefix.is_empty() { "" } else if is_last { "    " } else { "│   " });
                for (i, e) in items.iter().enumerate() {
                    Self::print_tree(&e.path(), &child_prefix, i == items.len() - 1, out);
                }
            }
        }
    }

    fn dir_size(path: &std::path::Path) -> u64 {
        if path.is_file() {
            return path.metadata().map(|m| m.len()).unwrap_or(0);
        }
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for e in entries.flatten() { total += Self::dir_size(&e.path()); }
        }
        total
    }

    fn human_size(bytes: u64) -> String {
        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit = 0;
        while size >= 1024.0 && unit < 4 { size /= 1024.0; unit += 1; }
        if unit == 0 { format!("{} {}", bytes, UNITS[unit]) }
        else { format!("{:.1} {}", size, UNITS[unit]) }
    }

    fn write_help(&self, out: &mut impl Write) {
        let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 the nestea shell\n\n\
\x1b[36mNavigation\x1b[0m    cd ls ll pwd tree find which\n\
\x1b[36mFiles\x1b[0m         cat head tail wc touch mkdir rm cp mv hex size\n\
\x1b[36mText\x1b[0m          echo grep\n\
\x1b[36mSystem\x1b[0m        whoami hostname os env time date open clear\n\
\x1b[36mShell\x1b[0m         let set unset export alias history run\n\
\x1b[36mMath\x1b[0m          calc <expr>\n\
\x1b[36mFlow\x1b[0m          cmd1 && cmd2    cmd > file    cmd >> file    cmd | cmd\n\
\x1b[36mOther\x1b[0m         Any unknown command runs as a system command\n\
\x1b[36mExit\x1b[0m          exit quit\n");
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
        let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 the nestea shell v3.0\n\
  nes <command>       run a single command\n\
  nes enter-full      launch interactive shell\n\
  nes run <file.nes>  run a script\n\
  nes --completions   list all commands\n\
  nes help            show all commands\n");
        let _ = out.flush();
        return;
    }
    let first = &args[0];
    if first == "--completions" {
        print!("cd\nls\nll\npwd\ntree\nfind\nwhich\ncat\nhead\ntail\nwc\ntouch\nmkdir\nrm\ncp\nmv\nhex\nsize\n\
echo\ngrep\nwhoami\nhostname\nos\nenv\ntime\ndate\nopen\nclear\ncls\n\
let\nset\nunset\nexport\nalias\nhistory\nrun\ncalc\nhelp\nenter-full\nexit\nquit\n");
        return;
    }
    let mut shell = Shell::new();
    if first == "enter-full" {
        let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
        let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 the nestea shell v3.0\n\n");
        let _ = out.flush();
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut buf = String::with_capacity(256);
        loop {
            {
                let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
                shell.prompt(&mut out);
            }
            buf.clear();
            if reader.read_line(&mut buf).unwrap_or(0) == 0 { break; }
            let input = buf.trim().to_string();
            if input.is_empty() { continue; }
            shell.history.push(input.clone());
            let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
            shell.exec(&input, &mut out);
            let _ = out.flush();
            if !shell.running { break; }
        }
    } else {
        let joined = args.join(" ");
        let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
        shell.exec(&joined, &mut out);
        let _ = out.flush();
    }
}

fn unix_secs() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

fn unix_to_datetime(mut secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    secs += 3600;
    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;
    let mut days = secs / 86400;
    let mut year = 1970u64;
    loop {
        let dy = 365 + is_leap(year) as u64;
        if days < dy { break; }
        days -= dy;
        year += 1;
    }
    let md: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &d in &md {
        if days < d { break; }
        days -= d;
        month += 1;
    }
    (year, month, days + 1, hour, min, sec)
}

fn is_leap(y: u64) -> bool { (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 }

fn eval_expr(expr: &str) -> Result<f64, &'static str> {
    let tokens = tokenize(expr.as_bytes())?;
    let mut pos = 0;
    let result = parse_add_sub(&tokens, &mut pos)?;
    if pos < tokens.len() { return Err("unexpected token"); }
    Ok(result)
}

#[derive(Clone, Copy)]
enum Token { Num(f64), Op(u8), LParen, RParen }

fn tokenize(b: &[u8]) -> Result<Vec<Token>, &'static str> {
    let mut t = Vec::with_capacity(32);
    let (mut i, len) = (0, b.len());
    while i < len {
        match b[i] {
            b' ' => i += 1,
            b'(' => { t.push(Token::LParen); i += 1; }
            b')' => { t.push(Token::RParen); i += 1; }
            b'+' | b'-' | b'*' | b'/' | b'%' | b'^' => { t.push(Token::Op(b[i])); i += 1; }
            b'0'..=b'9' | b'.' => {
                let s = i;
                while i < len && (b[i].is_ascii_digit() || b[i] == b'.') { i += 1; }
                let n = unsafe { std::str::from_utf8_unchecked(&b[s..i]) }
                    .parse::<f64>().map_err(|_| "bad number")?;
                t.push(Token::Num(n));
            }
            _ => return Err("unexpected char"),
        }
    }
    Ok(t)
}

fn parse_add_sub(t: &[Token], p: &mut usize) -> Result<f64, &'static str> {
    let mut l = parse_mul_div(t, p)?;
    while *p < t.len() {
        match t[*p] {
            Token::Op(b'+') => { *p += 1; l += parse_mul_div(t, p)?; }
            Token::Op(b'-') => { *p += 1; l -= parse_mul_div(t, p)?; }
            _ => break,
        }
    }
    Ok(l)
}

fn parse_mul_div(t: &[Token], p: &mut usize) -> Result<f64, &'static str> {
    let mut l = parse_power(t, p)?;
    while *p < t.len() {
        match t[*p] {
            Token::Op(b'*') => { *p += 1; l *= parse_power(t, p)?; }
            Token::Op(b'/') => { *p += 1; let r = parse_power(t, p)?; if r == 0.0 { return Err("div/0"); } l /= r; }
            Token::Op(b'%') => { *p += 1; let r = parse_power(t, p)?; if r == 0.0 { return Err("mod/0"); } l %= r; }
            _ => break,
        }
    }
    Ok(l)
}

fn parse_power(t: &[Token], p: &mut usize) -> Result<f64, &'static str> {
    let base = parse_unary(t, p)?;
    if *p < t.len() { if let Token::Op(b'^') = t[*p] { *p += 1; let exp = parse_power(t, p)?; return Ok(base.powf(exp)); } }
    Ok(base)
}

fn parse_unary(t: &[Token], p: &mut usize) -> Result<f64, &'static str> {
    if *p < t.len() {
        if let Token::Op(b'-') = t[*p] { *p += 1; return Ok(-parse_primary(t, p)?); }
        if let Token::Op(b'+') = t[*p] { *p += 1; return parse_primary(t, p); }
    }
    parse_primary(t, p)
}

fn parse_primary(t: &[Token], p: &mut usize) -> Result<f64, &'static str> {
    if *p >= t.len() { return Err("unexpected end"); }
    match t[*p] {
        Token::Num(n) => { *p += 1; Ok(n) }
        Token::LParen => {
            *p += 1;
            let v = parse_add_sub(t, p)?;
            if *p >= t.len() { return Err("missing )"); }
            if let Token::RParen = t[*p] { *p += 1; Ok(v) } else { Err("expected )") }
        }
        _ => Err("unexpected token"),
    }
}
