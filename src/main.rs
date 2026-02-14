use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

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

    fn block_prompt(out: &mut impl Write) {
        let _ = write!(out, "\x1b[33m ...>\x1b[0m ");
        let _ = out.flush();
    }

    // ── Block execution (if/for/comments) ─────────────────────────

    fn exec_lines(&mut self, lines: &[String], out: &mut BufWriter<io::StdoutLock<'_>>) {
        let mut pc = 0;
        while pc < lines.len() && self.running {
            let raw = lines[pc].trim();
            if raw.is_empty() || raw.starts_with('#') { pc += 1; continue; }

            if raw.starts_with("if ") {
                let (else_idx, end_idx) = Self::find_block_end(lines, pc);
                if end_idx >= lines.len() {
                    let _ = write!(out, "\x1b[31mnes: missing 'end' for 'if'\x1b[0m\n");
                    return;
                }
                let cond = self.expand_vars(&raw[3..]);
                if self.eval_condition(&cond) {
                    let stop = else_idx.unwrap_or(end_idx);
                    let body: Vec<String> = lines[pc + 1..stop].to_vec();
                    self.exec_lines(&body, out);
                } else if let Some(ei) = else_idx {
                    let body: Vec<String> = lines[ei + 1..end_idx].to_vec();
                    self.exec_lines(&body, out);
                }
                pc = end_idx + 1;
            } else if raw.starts_with("for ") {
                let (_, end_idx) = Self::find_block_end(lines, pc);
                if end_idx >= lines.len() {
                    let _ = write!(out, "\x1b[31mnes: missing 'end' for 'for'\x1b[0m\n");
                    return;
                }
                let body: Vec<String> = lines[pc + 1..end_idx].to_vec();
                let header = self.expand_vars(raw);
                self.exec_for(&header, &body, out);
                pc = end_idx + 1;
            } else {
                self.exec(raw, out);
                pc += 1;
            }
        }
    }

    fn find_block_end(lines: &[String], start: usize) -> (Option<usize>, usize) {
        let mut depth = 0u32;
        let mut else_pos = None;
        for i in (start + 1)..lines.len() {
            let l = lines[i].trim();
            if l.starts_with("if ") || l.starts_with("for ") { depth += 1; }
            else if l == "end" {
                if depth == 0 { return (else_pos, i); }
                depth -= 1;
            } else if l == "else" && depth == 0 {
                else_pos = Some(i);
            }
        }
        (None, lines.len()) // no matching end
    }

    fn eval_condition(&self, cond: &str) -> bool {
        let cond = cond.trim();
        if let Some(rest) = cond.strip_prefix("exists ") {
            return Path::new(rest.trim()).exists();
        }
        if let Some(rest) = cond.strip_prefix("not ") {
            return !self.eval_condition(rest);
        }
        if let Some(pos) = cond.find(" >= ") {
            let l: f64 = cond[..pos].trim().parse().unwrap_or(f64::NAN);
            let r: f64 = cond[pos + 4..].trim().parse().unwrap_or(f64::NAN);
            return l >= r;
        }
        if let Some(pos) = cond.find(" <= ") {
            let l: f64 = cond[..pos].trim().parse().unwrap_or(f64::NAN);
            let r: f64 = cond[pos + 4..].trim().parse().unwrap_or(f64::NAN);
            return l <= r;
        }
        if let Some(pos) = cond.find(" == ") {
            return cond[..pos].trim() == cond[pos + 4..].trim();
        }
        if let Some(pos) = cond.find(" != ") {
            return cond[..pos].trim() != cond[pos + 4..].trim();
        }
        if let Some(pos) = cond.find(" > ") {
            let l: f64 = cond[..pos].trim().parse().unwrap_or(f64::NAN);
            let r: f64 = cond[pos + 3..].trim().parse().unwrap_or(f64::NAN);
            return l > r;
        }
        if let Some(pos) = cond.find(" < ") {
            let l: f64 = cond[..pos].trim().parse().unwrap_or(f64::NAN);
            let r: f64 = cond[pos + 3..].trim().parse().unwrap_or(f64::NAN);
            return l < r;
        }
        !cond.is_empty() && cond != "false" && cond != "0"
    }

    fn exec_for(&mut self, header: &str, body: &[String], out: &mut BufWriter<io::StdoutLock<'_>>) {
        let after = header[4..].trim();
        let (var, rest) = match after.find(" in ") {
            Some(p) => (after[..p].trim(), after[p + 4..].trim()),
            None => return,
        };
        let var = var.to_string();
        let items: Vec<String> = if rest == "files" || rest.starts_with("files ") {
            let dir = if rest == "files" { "." } else { rest[5..].trim() };
            let dir = if dir.is_empty() { "." } else { dir };
            let mut v: Vec<String> = fs::read_dir(dir).ok()
                .map(|e| e.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
                .unwrap_or_default();
            v.sort_unstable();
            v
        } else if rest.starts_with("range ") {
            let p: Vec<&str> = rest[6..].split_whitespace().collect();
            if p.len() >= 2 {
                let s: i64 = p[0].parse().unwrap_or(0);
                let e: i64 = p[1].parse().unwrap_or(0);
                if s <= e { (s..=e).map(|n| n.to_string()).collect() }
                else { (e..=s).rev().map(|n| n.to_string()).collect() }
            } else { Vec::new() }
        } else if rest.starts_with("lines ") {
            let file = rest[6..].trim();
            fs::read_to_string(file).ok()
                .map(|c| c.lines().map(String::from).collect())
                .unwrap_or_default()
        } else {
            Self::split_args(rest)
        };
        for item in items {
            if !self.running { break; }
            self.vars.insert(var.clone(), item);
            self.exec_lines(body, out);
        }
    }

    // ── Single-line execution (&&, |, >, >>) ─────────────────────

    fn exec(&mut self, raw: &str, out: &mut BufWriter<io::StdoutLock<'_>>) {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('#') { return; }
        if raw == "end" || raw == "else" { return; }
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
            if chars[i] == '$' && i + 1 < chars.len() && (chars[i + 1].is_alphanumeric() || chars[i + 1] == '_') {
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

    // ── Command dispatch ──────────────────────────────────────────

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
            "read" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: read <varname>\n"); return; }
                let _ = out.flush();
                let mut input = String::new();
                let _ = io::stdin().read_line(&mut input);
                self.vars.insert(arg_str.clone(), input.trim().to_string());
            }
            "sleep" => {
                let ms: u64 = arg_str.trim().parse().unwrap_or(0);
                if ms > 0 { thread::sleep(Duration::from_millis(ms)); }
            }
            "exists" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: exists <path>\n"); return; }
                let _ = write!(out, "{}\n", Path::new(arg_str.as_str()).exists());
            }
            "count" => {
                let dir = if arg_str.is_empty() { ".".into() } else { arg_str };
                let n = fs::read_dir(&dir).ok().map(|e| e.count()).unwrap_or(0);
                let _ = write!(out, "{}\n", n);
            }
            "typeof" => {
                if arg_str.is_empty() { let _ = out.write_all(b"Usage: typeof <path>\n"); return; }
                let p = Path::new(arg_str.as_str());
                if p.is_file() { let _ = out.write_all(b"file\n"); }
                else if p.is_dir() { let _ = out.write_all(b"dir\n"); }
                else { let _ = out.write_all(b"none\n"); }
            }
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
                let path = Path::new(arg_str.as_str());
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
                Self::find_recursive(Path::new("."), pattern, out);
            }
            "tree" => {
                let dir = if arg_str.is_empty() { "." } else { &arg_str };
                Self::print_tree(Path::new(dir), "", true, out);
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
                    let lines: Vec<String> = script.lines().map(String::from).collect();
                    self.exec_lines(&lines, out);
                } else {
                    let _ = write!(out, "run: cannot read '{}'\n", arg_str);
                }
            }
            "which" => {
                if let Ok(path) = env::var("PATH") {
                    for dir in path.split(';') {
                        let p = Path::new(dir).join(format!("{}.exe", arg_str));
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
                let path = Path::new(arg_str.as_str());
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

    // ── Helpers ───────────────────────────────────────────────────

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

    fn find_recursive(dir: &Path, pattern: &str, out: &mut impl Write) {
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

    fn print_tree(dir: &Path, prefix: &str, is_last: bool, out: &mut impl Write) {
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

    fn dir_size(path: &Path) -> u64 {
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
        let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 v5.0\n\n\
\x1b[33m NesC (Shell)\x1b[0m\n\
\x1b[36mNavigation\x1b[0m    cd ls ll pwd tree find which\n\
\x1b[36mFiles\x1b[0m         cat head tail wc touch mkdir rm cp mv hex size\n\
\x1b[36mText\x1b[0m          echo grep\n\
\x1b[36mSystem\x1b[0m        whoami hostname os env time date open clear\n\
\x1b[36mShell\x1b[0m         let set unset export alias history run read\n\
\x1b[36mControl\x1b[0m       if/else/end  for/end  sleep  exists  count  typeof\n\
\x1b[36mMath\x1b[0m          calc <expr>\n\
\x1b[36mFlow\x1b[0m          cmd1 && cmd2    cmd > file    cmd >> file    cmd | cmd\n\
\x1b[36mOther\x1b[0m         Any unknown command runs as a system command\n\
\x1b[36mExit\x1b[0m          exit quit\n\n\
\x1b[33m NesT (Language)\x1b[0m    nes run <file.nest>\n\
\x1b[36mTypes\x1b[0m         int  float  str  bool\n\
\x1b[36mSyntax\x1b[0m        let x = 5;  x = x + 1;  fn name(a, b) { }\n\
\x1b[36mControl\x1b[0m       if/else { }  for i in 0..10 { }  while cond { }\n\
\x1b[36mI/O\x1b[0m           print()  println()  input()\n\
\x1b[36mBuilt-ins\x1b[0m     len() type() str() int() float() abs() sqrt() min() max() pow()\n\
\x1b[36mOperators\x1b[0m     + - * / %  == != < > <= >=  && || !\n\
\x1b[36mOther\x1b[0m         return  break  continue  # comments  // comments\n");
    }
}

// ══════════════════════════════════════════════════════════════════
// NesT — The Nes Programming Language (.nest files)
// ══════════════════════════════════════════════════════════════════

#[derive(Clone)]
enum NVal {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
}

impl NVal {
    fn is_truthy(&self) -> bool {
        match self {
            NVal::Bool(b) => *b,
            NVal::Int(n) => *n != 0,
            NVal::Float(n) => *n != 0.0,
            NVal::Str(s) => !s.is_empty(),
            NVal::None => false,
        }
    }
    fn as_f64(&self) -> f64 {
        match self {
            NVal::Int(n) => *n as f64,
            NVal::Float(n) => *n,
            NVal::Bool(b) => if *b { 1.0 } else { 0.0 },
            _ => f64::NAN,
        }
    }
    fn type_name(&self) -> &'static str {
        match self { NVal::Int(_) => "int", NVal::Float(_) => "float",
            NVal::Str(_) => "str", NVal::Bool(_) => "bool", NVal::None => "none" }
    }
}

impl std::fmt::Display for NVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NVal::Int(n) => write!(f, "{}", n),
            NVal::Float(n) => write!(f, "{}", n),
            NVal::Str(s) => write!(f, "{}", s),
            NVal::Bool(b) => write!(f, "{}", b),
            NVal::None => write!(f, "none"),
        }
    }
}

// ── NesT Tokens ───────────────────────────────────────────────

#[derive(Clone, Debug)]
enum NTok {
    IntLit(i64), FloatLit(f64), StrLit(String), BoolLit(bool),
    Ident(String),
    Let, Fn, If, Else, For, While, In, Return, Break, Continue,
    Plus, Minus, Star, Slash, Pct,
    EqEq, BangEq, Lt, Gt, LtEq, GtEq, AmpAmp, PipePipe, Bang,
    Eq, LParen, RParen, LBrace, RBrace, Comma, Semi, DotDot,
    Eof,
}

// ── NesT Lexer ────────────────────────────────────────────────

fn nest_tokenize(src: &str) -> Result<Vec<NTok>, String> {
    let b = src.as_bytes();
    let mut tokens = Vec::with_capacity(128);
    let (mut i, len) = (0usize, b.len());
    while i < len {
        match b[i] {
            b' ' | b'\t' | b'\r' | b'\n' => i += 1,
            b'#' => { while i < len && b[i] != b'\n' { i += 1; } }
            b'/' if i + 1 < len && b[i + 1] == b'/' => { while i < len && b[i] != b'\n' { i += 1; } }
            b'(' => { tokens.push(NTok::LParen); i += 1; }
            b')' => { tokens.push(NTok::RParen); i += 1; }
            b'{' => { tokens.push(NTok::LBrace); i += 1; }
            b'}' => { tokens.push(NTok::RBrace); i += 1; }
            b',' => { tokens.push(NTok::Comma); i += 1; }
            b';' => { tokens.push(NTok::Semi); i += 1; }
            b'+' => { tokens.push(NTok::Plus); i += 1; }
            b'-' => { tokens.push(NTok::Minus); i += 1; }
            b'*' => { tokens.push(NTok::Star); i += 1; }
            b'/' => { tokens.push(NTok::Slash); i += 1; }
            b'%' => { tokens.push(NTok::Pct); i += 1; }
            b'.' if i + 1 < len && b[i + 1] == b'.' => { tokens.push(NTok::DotDot); i += 2; }
            b'=' if i + 1 < len && b[i + 1] == b'=' => { tokens.push(NTok::EqEq); i += 2; }
            b'=' => { tokens.push(NTok::Eq); i += 1; }
            b'!' if i + 1 < len && b[i + 1] == b'=' => { tokens.push(NTok::BangEq); i += 2; }
            b'!' => { tokens.push(NTok::Bang); i += 1; }
            b'<' if i + 1 < len && b[i + 1] == b'=' => { tokens.push(NTok::LtEq); i += 2; }
            b'<' => { tokens.push(NTok::Lt); i += 1; }
            b'>' if i + 1 < len && b[i + 1] == b'=' => { tokens.push(NTok::GtEq); i += 2; }
            b'>' => { tokens.push(NTok::Gt); i += 1; }
            b'&' if i + 1 < len && b[i + 1] == b'&' => { tokens.push(NTok::AmpAmp); i += 2; }
            b'|' if i + 1 < len && b[i + 1] == b'|' => { tokens.push(NTok::PipePipe); i += 2; }
            b'"' => {
                i += 1;
                let mut s = String::new();
                while i < len && b[i] != b'"' {
                    if b[i] == b'\\' && i + 1 < len {
                        i += 1;
                        match b[i] {
                            b'n' => s.push('\n'),
                            b't' => s.push('\t'),
                            b'\\' => s.push('\\'),
                            b'"' => s.push('"'),
                            _ => { s.push('\\'); s.push(b[i] as char); }
                        }
                    } else { s.push(b[i] as char); }
                    i += 1;
                }
                if i >= len { return Err("unterminated string".into()); }
                i += 1; // closing "
                tokens.push(NTok::StrLit(s));
            }
            b'0'..=b'9' => {
                let s = i;
                let mut is_float = false;
                while i < len && (b[i].is_ascii_digit() || b[i] == b'.') {
                    if b[i] == b'.' {
                        if i + 1 < len && b[i + 1] == b'.' { break; } // ".." operator
                        is_float = true;
                    }
                    i += 1;
                }
                let num_str = unsafe { std::str::from_utf8_unchecked(&b[s..i]) };
                if is_float {
                    tokens.push(NTok::FloatLit(num_str.parse().map_err(|_| "bad float")?));
                } else {
                    tokens.push(NTok::IntLit(num_str.parse().map_err(|_| "bad int")?));
                }
            }
            c if c.is_ascii_alphabetic() || c == b'_' => {
                let s = i;
                while i < len && (b[i].is_ascii_alphanumeric() || b[i] == b'_') { i += 1; }
                let word = unsafe { std::str::from_utf8_unchecked(&b[s..i]) };
                tokens.push(match word {
                    "let" => NTok::Let, "fn" => NTok::Fn, "if" => NTok::If,
                    "else" => NTok::Else, "for" => NTok::For, "while" => NTok::While,
                    "in" => NTok::In, "return" => NTok::Return, "break" => NTok::Break,
                    "continue" => NTok::Continue,
                    "true" => NTok::BoolLit(true), "false" => NTok::BoolLit(false),
                    _ => NTok::Ident(word.to_string()),
                });
            }
            _ => return Err(format!("unexpected char '{}'", b[i] as char)),
        }
    }
    tokens.push(NTok::Eof);
    Ok(tokens)
}

// ── NesT AST ──────────────────────────────────────────────────

#[derive(Clone)]
enum NExpr {
    Lit(NVal),
    Var(String),
    Bin(Box<NExpr>, NBinOp, Box<NExpr>),
    Un(NUnOp, Box<NExpr>),
    Call(String, Vec<NExpr>),
}

#[derive(Copy, Clone)]
enum NBinOp { Add, Sub, Mul, Div, Mod, Eq, Ne, Lt, Gt, Le, Ge, And, Or }
#[derive(Copy, Clone)]
enum NUnOp { Neg, Not }

#[derive(Clone)]
enum NStmt {
    Dir(String, NVal),
    Let(String, NExpr),
    Assign(String, NExpr),
    If(NExpr, Vec<NStmt>, Option<Vec<NStmt>>),
    For(String, NExpr, NExpr, Vec<NStmt>),
    While(NExpr, Vec<NStmt>),
    FnDef(String, Vec<String>, Vec<NStmt>),
    Return(Option<NExpr>),
    Break,
    Continue,
    ExprStmt(NExpr),
}

// ── NesT Parser ───────────────────────────────────────────────

struct NParser { tokens: Vec<NTok>, pos: usize }

impl NParser {
    fn new(tokens: Vec<NTok>) -> Self { Self { tokens, pos: 0 } }
    fn peek(&self) -> &NTok { &self.tokens[self.pos.min(self.tokens.len() - 1)] }
    fn advance(&mut self) -> NTok { let t = self.tokens[self.pos].clone(); self.pos += 1; t }
    fn expect_semi(&mut self) -> Result<(), String> {
        if matches!(self.peek(), NTok::Semi) { self.advance(); Ok(()) }
        else { Err("expected ';'".into()) }
    }
    fn at_eof(&self) -> bool { matches!(self.peek(), NTok::Eof) }

    fn parse_program(&mut self) -> Result<Vec<NStmt>, String> {
        let mut stmts = Vec::new();
        while !self.at_eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<NStmt, String> {
        match self.peek().clone() {
            NTok::Let => self.parse_let(),
            NTok::Fn => self.parse_fn_def(),
            NTok::If => self.parse_if(),
            NTok::For => self.parse_for(),
            NTok::While => self.parse_while(),
            NTok::Return => {
                self.advance();
                let expr = if matches!(self.peek(), NTok::Semi | NTok::RBrace) { None }
                    else { Some(self.parse_expr()?) };
                self.expect_semi()?;
                Ok(NStmt::Return(expr))
            }
            NTok::Break => { self.advance(); self.expect_semi()?; Ok(NStmt::Break) }
            NTok::Continue => { self.advance(); self.expect_semi()?; Ok(NStmt::Continue) }
            NTok::Ident(_) => {
                // Could be directive (ident = value;) or assignment (ident = expr;)
                // or expression statement (fn call etc)
                let name = if let NTok::Ident(n) = self.advance() { n } else { unreachable!() };
                if matches!(self.peek(), NTok::Eq) {
                    self.advance(); // consume =
                    let expr = self.parse_expr()?;
                    self.expect_semi()?;
                    // Check if it's a directive (top-level simple value assignment to unset var)
                    Ok(NStmt::Assign(name, expr))
                } else {
                    // Put ident back as a call expression or standalone
                    self.pos -= 1;
                    let expr = self.parse_expr()?;
                    self.expect_semi()?;
                    Ok(NStmt::ExprStmt(expr))
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect_semi()?;
                Ok(NStmt::ExprStmt(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<NStmt, String> {
        self.advance(); // let
        let name = match self.advance() {
            NTok::Ident(n) => n,
            _ => return Err("expected variable name after 'let'".into()),
        };
        if !matches!(self.peek(), NTok::Eq) { return Err("expected '=' in let".into()); }
        self.advance();
        let expr = self.parse_expr()?;
        self.expect_semi()?;
        Ok(NStmt::Let(name, expr))
    }

    fn parse_fn_def(&mut self) -> Result<NStmt, String> {
        self.advance(); // fn
        let name = match self.advance() {
            NTok::Ident(n) => n,
            _ => return Err("expected function name".into()),
        };
        if !matches!(self.peek(), NTok::LParen) { return Err("expected '(' after fn name".into()); }
        self.advance();
        let mut params = Vec::new();
        while !matches!(self.peek(), NTok::RParen) {
            match self.advance() {
                NTok::Ident(p) => params.push(p),
                _ => return Err("expected param name".into()),
            }
            if matches!(self.peek(), NTok::Comma) { self.advance(); }
        }
        self.advance(); // )
        let body = self.parse_block()?;
        Ok(NStmt::FnDef(name, params, body))
    }

    fn parse_if(&mut self) -> Result<NStmt, String> {
        self.advance(); // if
        let cond = self.parse_expr()?;
        let then_body = self.parse_block()?;
        let else_body = if matches!(self.peek(), NTok::Else) {
            self.advance();
            if matches!(self.peek(), NTok::If) {
                Some(vec![self.parse_if()?])
            } else {
                Some(self.parse_block()?)
            }
        } else { None };
        Ok(NStmt::If(cond, then_body, else_body))
    }

    fn parse_for(&mut self) -> Result<NStmt, String> {
        self.advance(); // for
        let var = match self.advance() {
            NTok::Ident(n) => n,
            _ => return Err("expected variable name in for".into()),
        };
        if !matches!(self.peek(), NTok::In) { return Err("expected 'in' in for".into()); }
        self.advance();
        let start = self.parse_expr()?;
        if !matches!(self.peek(), NTok::DotDot) { return Err("expected '..' in for range".into()); }
        self.advance();
        let end = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(NStmt::For(var, start, end, body))
    }

    fn parse_while(&mut self) -> Result<NStmt, String> {
        self.advance(); // while
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(NStmt::While(cond, body))
    }

    fn parse_block(&mut self) -> Result<Vec<NStmt>, String> {
        if !matches!(self.peek(), NTok::LBrace) { return Err("expected '{'".into()); }
        self.advance();
        let mut stmts = Vec::new();
        while !matches!(self.peek(), NTok::RBrace) {
            if self.at_eof() { return Err("unexpected EOF, expected '}'".into()); }
            stmts.push(self.parse_stmt()?);
        }
        self.advance(); // }
        Ok(stmts)
    }

    // ── Expression parsing (precedence climbing) ──────────────

    fn parse_expr(&mut self) -> Result<NExpr, String> { self.parse_or() }

    fn parse_or(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_and()?;
        while matches!(self.peek(), NTok::PipePipe) {
            self.advance(); let r = self.parse_and()?;
            l = NExpr::Bin(Box::new(l), NBinOp::Or, Box::new(r));
        }
        Ok(l)
    }

    fn parse_and(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_equality()?;
        while matches!(self.peek(), NTok::AmpAmp) {
            self.advance(); let r = self.parse_equality()?;
            l = NExpr::Bin(Box::new(l), NBinOp::And, Box::new(r));
        }
        Ok(l)
    }

    fn parse_equality(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_comparison()?;
        loop {
            match self.peek() {
                NTok::EqEq => { self.advance(); let r = self.parse_comparison()?; l = NExpr::Bin(Box::new(l), NBinOp::Eq, Box::new(r)); }
                NTok::BangEq => { self.advance(); let r = self.parse_comparison()?; l = NExpr::Bin(Box::new(l), NBinOp::Ne, Box::new(r)); }
                _ => break,
            }
        }
        Ok(l)
    }

    fn parse_comparison(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_add()?;
        loop {
            match self.peek() {
                NTok::Lt => { self.advance(); let r = self.parse_add()?; l = NExpr::Bin(Box::new(l), NBinOp::Lt, Box::new(r)); }
                NTok::Gt => { self.advance(); let r = self.parse_add()?; l = NExpr::Bin(Box::new(l), NBinOp::Gt, Box::new(r)); }
                NTok::LtEq => { self.advance(); let r = self.parse_add()?; l = NExpr::Bin(Box::new(l), NBinOp::Le, Box::new(r)); }
                NTok::GtEq => { self.advance(); let r = self.parse_add()?; l = NExpr::Bin(Box::new(l), NBinOp::Ge, Box::new(r)); }
                _ => break,
            }
        }
        Ok(l)
    }

    fn parse_add(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_mul()?;
        loop {
            match self.peek() {
                NTok::Plus => { self.advance(); let r = self.parse_mul()?; l = NExpr::Bin(Box::new(l), NBinOp::Add, Box::new(r)); }
                NTok::Minus => { self.advance(); let r = self.parse_mul()?; l = NExpr::Bin(Box::new(l), NBinOp::Sub, Box::new(r)); }
                _ => break,
            }
        }
        Ok(l)
    }

    fn parse_mul(&mut self) -> Result<NExpr, String> {
        let mut l = self.parse_unary()?;
        loop {
            match self.peek() {
                NTok::Star => { self.advance(); let r = self.parse_unary()?; l = NExpr::Bin(Box::new(l), NBinOp::Mul, Box::new(r)); }
                NTok::Slash => { self.advance(); let r = self.parse_unary()?; l = NExpr::Bin(Box::new(l), NBinOp::Div, Box::new(r)); }
                NTok::Pct => { self.advance(); let r = self.parse_unary()?; l = NExpr::Bin(Box::new(l), NBinOp::Mod, Box::new(r)); }
                _ => break,
            }
        }
        Ok(l)
    }

    fn parse_unary(&mut self) -> Result<NExpr, String> {
        match self.peek() {
            NTok::Minus => { self.advance(); let e = self.parse_unary()?; Ok(NExpr::Un(NUnOp::Neg, Box::new(e))) }
            NTok::Bang => { self.advance(); let e = self.parse_unary()?; Ok(NExpr::Un(NUnOp::Not, Box::new(e))) }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<NExpr, String> {
        match self.peek().clone() {
            NTok::IntLit(n) => { self.advance(); Ok(NExpr::Lit(NVal::Int(n))) }
            NTok::FloatLit(n) => { self.advance(); Ok(NExpr::Lit(NVal::Float(n))) }
            NTok::StrLit(s) => { self.advance(); Ok(NExpr::Lit(NVal::Str(s))) }
            NTok::BoolLit(b) => { self.advance(); Ok(NExpr::Lit(NVal::Bool(b))) }
            NTok::Ident(name) => {
                self.advance();
                if matches!(self.peek(), NTok::LParen) {
                    self.advance(); // (
                    let mut args = Vec::new();
                    while !matches!(self.peek(), NTok::RParen) {
                        args.push(self.parse_expr()?);
                        if matches!(self.peek(), NTok::Comma) { self.advance(); }
                    }
                    self.advance(); // )
                    Ok(NExpr::Call(name, args))
                } else {
                    Ok(NExpr::Var(name))
                }
            }
            NTok::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                if !matches!(self.peek(), NTok::RParen) { return Err("expected ')'".into()); }
                self.advance();
                Ok(e)
            }
            _ => Err(format!("unexpected token in expression: {:?}", self.peek())),
        }
    }
}

// ── NesT Interpreter ──────────────────────────────────────────

enum NFlow { None, Return(NVal), Break, Continue }

struct NestRunner {
    vars: Vec<HashMap<String, NVal>>,
    fns: HashMap<String, (Vec<String>, Vec<NStmt>)>,
    directives: HashMap<String, NVal>,
}

impl NestRunner {
    fn new() -> Self {
        Self {
            vars: vec![HashMap::new()],
            fns: HashMap::new(),
            directives: HashMap::new(),
        }
    }

    fn set_var(&mut self, name: &str, val: NVal) {
        for scope in self.vars.iter_mut().rev() {
            if scope.contains_key(name) { scope.insert(name.to_string(), val); return; }
        }
        self.vars.last_mut().unwrap().insert(name.to_string(), val);
    }

    fn get_var(&self, name: &str) -> Result<NVal, String> {
        for scope in self.vars.iter().rev() {
            if let Some(v) = scope.get(name) { return Ok(v.clone()); }
        }
        Err(format!("undefined variable '{}'", name))
    }

    fn push_scope(&mut self) { self.vars.push(HashMap::new()); }
    fn pop_scope(&mut self) { if self.vars.len() > 1 { self.vars.pop(); } }

    fn run(&mut self, src: &str) -> Result<(), String> {
        let tokens = nest_tokenize(src)?;
        let mut parser = NParser::new(tokens);
        let stmts = parser.parse_program()?;
        self.exec_block(&stmts)?;
        Ok(())
    }

    fn exec_block(&mut self, stmts: &[NStmt]) -> Result<NFlow, String> {
        for stmt in stmts {
            let flow = self.exec_stmt(stmt)?;
            match flow {
                NFlow::None => {}
                other => return Ok(other),
            }
        }
        Ok(NFlow::None)
    }

    fn exec_stmt(&mut self, stmt: &NStmt) -> Result<NFlow, String> {
        match stmt {
            NStmt::Dir(name, val) => {
                self.directives.insert(name.clone(), val.clone());
            }
            NStmt::Let(name, expr) => {
                let val = self.eval(expr)?;
                self.vars.last_mut().unwrap().insert(name.clone(), val);
            }
            NStmt::Assign(name, expr) => {
                let val = self.eval(expr)?;
                self.set_var(name, val);
            }
            NStmt::If(cond, then_b, else_b) => {
                let val = self.eval(cond)?;
                self.push_scope();
                let flow = if val.is_truthy() {
                    self.exec_block(then_b)?
                } else if let Some(eb) = else_b {
                    self.exec_block(eb)?
                } else { NFlow::None };
                self.pop_scope();
                return Ok(flow);
            }
            NStmt::For(var, start, end, body) => {
                let s = match self.eval(start)? { NVal::Int(n) => n, v => return Err(format!("for range start must be int, got {}", v.type_name())) };
                let e = match self.eval(end)? { NVal::Int(n) => n, v => return Err(format!("for range end must be int, got {}", v.type_name())) };
                self.push_scope();
                if s <= e {
                    for i in s..e {
                        self.vars.last_mut().unwrap().insert(var.clone(), NVal::Int(i));
                        match self.exec_block(body)? {
                            NFlow::Break => break,
                            NFlow::Return(v) => { self.pop_scope(); return Ok(NFlow::Return(v)); }
                            NFlow::Continue | NFlow::None => {}
                        }
                    }
                } else {
                    for i in (e..s).rev() {
                        self.vars.last_mut().unwrap().insert(var.clone(), NVal::Int(i));
                        match self.exec_block(body)? {
                            NFlow::Break => break,
                            NFlow::Return(v) => { self.pop_scope(); return Ok(NFlow::Return(v)); }
                            NFlow::Continue | NFlow::None => {}
                        }
                    }
                }
                self.pop_scope();
            }
            NStmt::While(cond, body) => {
                self.push_scope();
                loop {
                    let c = self.eval(cond)?;
                    if !c.is_truthy() { break; }
                    match self.exec_block(body)? {
                        NFlow::Break => break,
                        NFlow::Return(v) => { self.pop_scope(); return Ok(NFlow::Return(v)); }
                        NFlow::Continue | NFlow::None => {}
                    }
                }
                self.pop_scope();
            }
            NStmt::FnDef(name, params, body) => {
                self.fns.insert(name.clone(), (params.clone(), body.clone()));
            }
            NStmt::Return(expr) => {
                let val = match expr {
                    Some(e) => self.eval(e)?,
                    Option::None => NVal::None,
                };
                return Ok(NFlow::Return(val));
            }
            NStmt::Break => return Ok(NFlow::Break),
            NStmt::Continue => return Ok(NFlow::Continue),
            NStmt::ExprStmt(expr) => { self.eval(expr)?; }
        }
        Ok(NFlow::None)
    }

    fn eval(&mut self, expr: &NExpr) -> Result<NVal, String> {
        match expr {
            NExpr::Lit(v) => Ok(v.clone()),
            NExpr::Var(name) => self.get_var(name),
            NExpr::Un(op, e) => {
                let v = self.eval(e)?;
                match op {
                    NUnOp::Neg => match v {
                        NVal::Int(n) => Ok(NVal::Int(-n)),
                        NVal::Float(n) => Ok(NVal::Float(-n)),
                        _ => Err("cannot negate non-number".into()),
                    },
                    NUnOp::Not => Ok(NVal::Bool(!v.is_truthy())),
                }
            }
            NExpr::Bin(l, op, r) => {
                let lv = self.eval(l)?;
                // Short-circuit for && and ||
                match op {
                    NBinOp::And => return Ok(if !lv.is_truthy() { lv } else { self.eval(r)? }),
                    NBinOp::Or => return Ok(if lv.is_truthy() { lv } else { self.eval(r)? }),
                    _ => {}
                }
                let rv = self.eval(r)?;
                match op {
                    NBinOp::Add => match (&lv, &rv) {
                        (NVal::Int(a), NVal::Int(b)) => Ok(NVal::Int(a + b)),
                        (NVal::Float(_), _) | (_, NVal::Float(_)) => Ok(NVal::Float(lv.as_f64() + rv.as_f64())),
                        (NVal::Str(a), _) => Ok(NVal::Str(format!("{}{}", a, rv))),
                        _ => Err(format!("cannot add {} + {}", lv.type_name(), rv.type_name())),
                    },
                    NBinOp::Sub => Ok(NVal::Float(lv.as_f64() - rv.as_f64())),
                    NBinOp::Mul => match (&lv, &rv) {
                        (NVal::Int(a), NVal::Int(b)) => Ok(NVal::Int(a * b)),
                        _ => Ok(NVal::Float(lv.as_f64() * rv.as_f64())),
                    },
                    NBinOp::Div => {
                        let d = rv.as_f64();
                        if d == 0.0 { return Err("division by zero".into()); }
                        Ok(NVal::Float(lv.as_f64() / d))
                    }
                    NBinOp::Mod => match (&lv, &rv) {
                        (NVal::Int(a), NVal::Int(b)) => { if *b == 0 { return Err("modulo by zero".into()); } Ok(NVal::Int(a % b)) }
                        _ => Ok(NVal::Float(lv.as_f64() % rv.as_f64())),
                    },
                    NBinOp::Eq => Ok(NVal::Bool(format!("{}", lv) == format!("{}", rv))),
                    NBinOp::Ne => Ok(NVal::Bool(format!("{}", lv) != format!("{}", rv))),
                    NBinOp::Lt => Ok(NVal::Bool(lv.as_f64() < rv.as_f64())),
                    NBinOp::Gt => Ok(NVal::Bool(lv.as_f64() > rv.as_f64())),
                    NBinOp::Le => Ok(NVal::Bool(lv.as_f64() <= rv.as_f64())),
                    NBinOp::Ge => Ok(NVal::Bool(lv.as_f64() >= rv.as_f64())),
                    NBinOp::And | NBinOp::Or => unreachable!(),
                }
            }
            NExpr::Call(name, args) => {
                let mut vals: Vec<NVal> = Vec::with_capacity(args.len());
                for a in args { vals.push(self.eval(a)?); }
                self.call_fn(name, vals)
            }
        }
    }

    fn call_fn(&mut self, name: &str, args: Vec<NVal>) -> Result<NVal, String> {
        // Built-in functions
        match name {
            "print" => {
                let s: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                print!("{}", s.join(" "));
                return Ok(NVal::None);
            }
            "println" => {
                let s: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                println!("{}", s.join(" "));
                return Ok(NVal::None);
            }
            "input" => {
                if let Some(prompt) = args.first() { print!("{}", prompt); }
                let _ = io::stdout().flush();
                let mut buf = String::new();
                let _ = io::stdin().read_line(&mut buf);
                return Ok(NVal::Str(buf.trim().to_string()));
            }
            "len" => {
                if args.len() != 1 { return Err("len() takes 1 argument".into()); }
                return match &args[0] {
                    NVal::Str(s) => Ok(NVal::Int(s.len() as i64)),
                    _ => Err("len() requires a string".into()),
                };
            }
            "type" => {
                if args.len() != 1 { return Err("type() takes 1 argument".into()); }
                return Ok(NVal::Str(args[0].type_name().to_string()));
            }
            "str" => {
                if args.len() != 1 { return Err("str() takes 1 argument".into()); }
                return Ok(NVal::Str(format!("{}", args[0])));
            }
            "int" => {
                if args.len() != 1 { return Err("int() takes 1 argument".into()); }
                return match &args[0] {
                    NVal::Int(n) => Ok(NVal::Int(*n)),
                    NVal::Float(n) => Ok(NVal::Int(*n as i64)),
                    NVal::Str(s) => s.trim().parse::<i64>().map(NVal::Int).map_err(|_| "cannot convert to int".into()),
                    NVal::Bool(b) => Ok(NVal::Int(if *b { 1 } else { 0 })),
                    NVal::None => Ok(NVal::Int(0)),
                };
            }
            "float" => {
                if args.len() != 1 { return Err("float() takes 1 argument".into()); }
                return match &args[0] {
                    NVal::Float(n) => Ok(NVal::Float(*n)),
                    NVal::Int(n) => Ok(NVal::Float(*n as f64)),
                    NVal::Str(s) => s.trim().parse::<f64>().map(NVal::Float).map_err(|_| "cannot convert to float".into()),
                    _ => Err("cannot convert to float".into()),
                };
            }
            "abs" => {
                if args.len() != 1 { return Err("abs() takes 1 argument".into()); }
                return match &args[0] {
                    NVal::Int(n) => Ok(NVal::Int(n.abs())),
                    NVal::Float(n) => Ok(NVal::Float(n.abs())),
                    _ => Err("abs() requires a number".into()),
                };
            }
            "sqrt" => {
                if args.len() != 1 { return Err("sqrt() takes 1 argument".into()); }
                return Ok(NVal::Float(args[0].as_f64().sqrt()));
            }
            "min" => {
                if args.len() != 2 { return Err("min() takes 2 arguments".into()); }
                let (a, b) = (args[0].as_f64(), args[1].as_f64());
                return Ok(NVal::Float(a.min(b)));
            }
            "max" => {
                if args.len() != 2 { return Err("max() takes 2 arguments".into()); }
                let (a, b) = (args[0].as_f64(), args[1].as_f64());
                return Ok(NVal::Float(a.max(b)));
            }
            "pow" => {
                if args.len() != 2 { return Err("pow() takes 2 arguments".into()); }
                return Ok(NVal::Float(args[0].as_f64().powf(args[1].as_f64())));
            }
            _ => {}
        }
        // User-defined functions
        if let Some((params, body)) = self.fns.get(name).cloned() {
            if args.len() != params.len() {
                return Err(format!("{}() expects {} args, got {}", name, params.len(), args.len()));
            }
            self.push_scope();
            for (p, v) in params.iter().zip(args) {
                self.vars.last_mut().unwrap().insert(p.clone(), v);
            }
            let flow = self.exec_block(&body)?;
            self.pop_scope();
            return match flow {
                NFlow::Return(v) => Ok(v),
                _ => Ok(NVal::None),
            };
        }
        Err(format!("undefined function '{}'", name))
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
        let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 the nestea shell v5.0\n\n\
  \x1b[36mNesC (shell)\x1b[0m\n\
  nes <command>         run a shell command\n\
  nes enter-full        interactive shell\n\
  nes run <file.nes>    run a NesC script\n\n\
  \x1b[36mNesT (language)\x1b[0m\n\
  nes run <file.nest>   run a NesT program\n\n\
  nes help              show all commands\n\
  nes --completions     list commands\n");
        let _ = out.flush();
        return;
    }
    let first = &args[0];
    if first == "--completions" {
        print!("cd\nls\nll\npwd\ntree\nfind\nwhich\ncat\nhead\ntail\nwc\ntouch\nmkdir\nrm\ncp\nmv\nhex\nsize\n\
echo\ngrep\nwhoami\nhostname\nos\nenv\ntime\ndate\nopen\nclear\ncls\n\
let\nset\nunset\nexport\nalias\nhistory\nrun\nread\nsleep\nexists\ncount\ntypeof\n\
if\nfor\nend\nelse\ncalc\nhelp\nenter-full\nexit\nquit\n");
        return;
    }
    // Check if running a .nest file
    if first == "run" && args.len() >= 2 && args[1].ends_with(".nest") {
        let mut runner = NestRunner::new();
        match fs::read_to_string(&args[1]) {
            Ok(src) => {
                if let Err(e) = runner.run(&src) {
                    eprintln!("\x1b[31mnest error:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => { eprintln!("nest: cannot read '{}': {}", args[1], e); std::process::exit(1); }
        }
        return;
    }
    // Also detect .nest when passed directly (nes myfile.nest)
    if first.ends_with(".nest") {
        let mut runner = NestRunner::new();
        match fs::read_to_string(first) {
            Ok(src) => {
                if let Err(e) = runner.run(&src) {
                    eprintln!("\x1b[31mnest error:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => { eprintln!("nest: cannot read '{}': {}", first, e); std::process::exit(1); }
        }
        return;
    }
    let mut shell = Shell::new();
    if first == "enter-full" {
        {
            let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
            let _ = out.write_all(b"\x1b[33mnes\x1b[0m \xE2\x80\x94 the nestea shell v5.0 (NesC + NesT)\n\n");
            let _ = out.flush();
        }
        let stdin = io::stdin();
        let mut buf = String::with_capacity(256);
        let mut block_buf: Vec<String> = Vec::new();
        let mut block_depth: i32 = 0;
        loop {
            {
                let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
                if block_depth > 0 { Shell::block_prompt(&mut out); }
                else { shell.prompt(&mut out); }
            }
            buf.clear();
            if stdin.read_line(&mut buf).unwrap_or(0) == 0 { break; }
            let input = buf.trim().to_string();
            if input.is_empty() { continue; }
            shell.history.push(input.clone());

            let trimmed = input.trim();
            let starts_block = trimmed.starts_with("if ") || trimmed.starts_with("for ");
            let is_end = trimmed == "end";

            if block_depth > 0 || starts_block {
                if starts_block { block_depth += 1; }
                block_buf.push(input);
                if is_end {
                    block_depth -= 1;
                    if block_depth == 0 {
                        let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
                        shell.exec_lines(&block_buf, &mut out);
                        let _ = out.flush();
                        block_buf.clear();
                    }
                }
            } else {
                let mut out = BufWriter::with_capacity(4096, io::stdout().lock());
                shell.exec(&input, &mut out);
                let _ = out.flush();
            }
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
