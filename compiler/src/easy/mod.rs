/// Y# Easy → Y# Standard transpiler
///
/// Converts simplified syntax (indentation-based, no {}, no ;, auto-string print)
/// into standard Y# for the compiler pipeline.
///
/// .yse syntax summary:
///   - No braces — Python-style indentation
///   - No semicolons — newline ends statement
///   - Single quotes '  →  double quotes (for non-print lines)
///   - print/println: text after keyword = auto-string
///   - `$name` in print = variable interpolation
///   - if/while conditions: no parens needed
///   - Implicit function calls: fn_name arg1 arg2
///   - Blank lines and // comments preserved

pub fn transpile(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = String::with_capacity(source.len() * 12 / 10);
    // Stack of indent levels that have open blocks.
    // Starts at 0 (global scope). When we see a block starter at indent N,
    // we push N to the stack. At dedent (new indent <= stack top), we emit }.
    let mut stack: Vec<usize> = vec![0];

    for raw_line in &lines {
        let trimmed = raw_line.trim();

        // Preserve blank lines exactly
        if trimmed.is_empty() {
            out.push_str(raw_line);
            out.push('\n');
            continue;
        }

        // Preserve comment lines
        if trimmed.starts_with("//") {
            out.push_str(raw_line);
            out.push('\n');
            continue;
        }

        let indent = raw_line.len() - trimmed.len();

        // --- Dedentation: close blocks when indent decreases or stays same as a block ---
        while stack.len() > 1 && indent <= *stack.last().unwrap() {
            let block_indent = stack.pop().unwrap();
            out.push_str(&" ".repeat(block_indent));
            out.push_str("}\n");
        }

        // --- Process the line content ---
        let processed = process_line(trimmed);

        // --- Block handling ---
        if is_block_starter(trimmed) {
            // This line introduces a block: fn, if, while, loop, for, etc.
            out.push_str(&" ".repeat(indent));
            out.push_str(&processed);
            out.push_str(" {");
            out.push('\n');
            stack.push(indent);
        } else if trimmed.starts_with("else") && !trimmed.starts_with("else if") {
            out.push_str(&" ".repeat(indent));
            out.push_str("else {");
            out.push('\n');
            stack.push(indent);
        } else {
            // Regular statement: emit with indent, add semicolon if needed
            out.push_str(&" ".repeat(indent));
            out.push_str(&processed);
            if !processed.ends_with(';') && !processed.ends_with('{') && !processed.ends_with('}') {
                out.push(';');
            }
            out.push('\n');
        }
    }

    // Close any remaining open blocks at end of file
    while stack.len() > 1 {
        stack.pop();
        let ci = *stack.last().unwrap();
        out.push_str(&" ".repeat(ci));
        out.push_str("}\n");
    }

    // Unwrap Function main() definitions — their body becomes Program body
    out = unwrap_main_body(&out);

    // Auto-wrap in Program Main { ... } if not already wrapped
    let trimmed = out.trim();
    if !trimmed.starts_with("Program ") && !trimmed.starts_with("program ") {
        let mut wrapped = String::with_capacity(out.len() + 50);
        wrapped.push_str("Program Main {\n");
        for line in out.lines() {
            if line.trim().is_empty() || line.trim().starts_with("//") {
                wrapped.push_str(line);
            } else {
                wrapped.push_str("    ");
                wrapped.push_str(line);
            }
            wrapped.push('\n');
        }
        wrapped.push_str("}\n");
        out = wrapped;
    }

    out
}

/// Scan for `Function main() { ... }` blocks and unwrap them,
/// placing their body directly at the definition site.
/// This avoids duplicate `main()` C functions.
fn unwrap_main_body(output: &str) -> String {
    let mut result = String::new();
    let mut in_main = false;
    let mut main_indent: Option<usize> = None;

    for line in output.lines() {
        let trimmed = line.trim();
        let indent = line.len() - trimmed.len();

        if trimmed == "Function main() {" {
            in_main = true;
            main_indent = Some(indent);
            continue;
        }

        if in_main {
            if trimmed == "}" && main_indent.map_or(false, |mi| indent == mi) {
                in_main = false;
                continue;
            }
            // Body line: remove one level of indent (4 spaces) relative to insertion point
            let body_line = if line.starts_with("    ") { &line[4..] } else { line };
            result.push_str(body_line);
            result.push('\n');
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }
    result
}

/// Check if a line introduces a block that needs `{`
fn is_block_starter(line: &str) -> bool {
    let t = line.trim();
    // Y# Easy block-introducing keywords (lowercase = Easy syntax)
    t.starts_with("fn ")
        || t.starts_with("if ")
        || t.starts_with("while ")
        || t.starts_with("loop ")
        || t.starts_with("for ")
        || t.starts_with("else if ")
        || t.starts_with("entity ")
        || t.starts_with("system ")
        || t.starts_with("actor ")
        || t.starts_with("on ")
        || t.starts_with("view ")
        || t.starts_with("state ")
}

/// Transform a single line of Y# Easy into standard Y#
fn process_line(line: &str) -> String {
    let line = line.trim();

    // print/println statements — handle BEFORE quote replacement
    if line == "print" { return "Print(\"\")".to_string(); }
    if line == "println" { return "PrintLine()".to_string(); }
    if line.starts_with("print ") || line.starts_with("println ") {
        return process_print(line);
    }

    // Replace single quotes '...' with "..."
    let line = replace_quotes(line);

    // Strip $ from variable names (outside strings)
    let line = strip_dollar_vars(&line);

    // Function definitions: 'fn ' → 'Function '
    if let Some(rest) = line.strip_prefix("fn ") {
        return process_fn_def(rest);
    }

    // if/while conditions: add parens (keywords are lowercase in parser)
    if let Some(cond) = line.strip_prefix("if ") {
        return format!("if ({})", cond);
    }
    if let Some(cond) = line.strip_prefix("while ") {
        return format!("while ({})", cond);
    }
    if let Some(cond) = line.strip_prefix("else if ") {
        return format!("else if ({})", cond);
    }
    if line == "else" {
        return "else".to_string();
    }

    // 'loop' → 'Loop(' with closing ')'
    if line.starts_with("loop ") {
        let rest = &line[5..].trim();
        return format!("Loop({})", rest);
    }

    // 'for': add parens if not present
    if let Some(rest) = line.strip_prefix("for ") {
        let rest = rest.trim();
        if !rest.starts_with('(') {
            return format!("for ({})", rest);
        }
        return line.to_string();
    }

    // 'return' → 'Return ' (trailing space required by Y# keyword_exact)
    if line.starts_with("return ") {
        return format!("Return {}", &line[7..]);
    }
    if line == "return" {
        return "Return ".to_string();
    }

    // Implicit function calls: name arg1 arg2 → name(arg1, arg2)
    if looks_like_call(&line) {
        return process_implicit_call(&line);
    }

    // Everything else: return as-is (var, let, const, expressions, etc.)
    line
}

/// Strip $ prefix from variable names outside of string literals.
/// Also used in process_fn_def to handle $param syntax.
fn strip_dollar_vars(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            // Look ahead: if followed by a valid identifier character, skip $
            // (effectively removing it)
            match chars.clone().next() {
                Some(next) if next.is_alphanumeric() || next == '_' => {
                    // $ was used as variable marker in Easy syntax
                    // Don't emit it - just skip to next iter which will emit the identifier char
                    continue;
                }
                _ => { out.push(c); }
            }
        } else if c == '"' {
            out.push(c);
            // Copy string literal content as-is (including any $ inside)
            loop {
                match chars.next() {
                    None => break,
                    Some('\\') => { out.push('\\'); if let Some(esc) = chars.next() { out.push(esc); } }
                    Some('"') => { out.push('"'); break; }
                    Some(c) => { out.push(c); }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Replace single-quoted strings with double-quoted ones.
/// Only processes text OUTSIDE of existing double quotes.
fn replace_quotes(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars();
    let mut in_dq = false;

    while let Some(c) = chars.next() {
        if c == '"' {
            in_dq = !in_dq;
            out.push(c);
        } else if c == '\'' && !in_dq {
            out.push('"');
            // Copy until closing quote
            loop {
                match chars.next() {
                    None => break,
                    Some('\'') => { out.push('"'); break; }
                    Some('\\') => {
                        match chars.next() {
                            None => out.push('\\'),
                            Some('n') => out.push_str("\\n"),
                            Some('t') => out.push_str("\\t"),
                            Some('r') => out.push_str("\\r"),
                            Some('0') => out.push_str("\\0"),
                            Some('\\') => out.push_str("\\\\"),
                            Some('\'') => out.push_str("'"),
                            Some('"') => out.push_str("\\\""),
                            Some(c) => { out.push('\\'); out.push(c); }
                        }
                    }
                    Some(c) => out.push(c),
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Transform fn definition: extract name, params, return type
fn process_fn_def(rest: &str) -> String {
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.is_empty() {
        return "fn ()".to_string();
    }

    // Check if already has parens (like fn name(params))
    let name = parts[0];
    if name.contains('(') && name.ends_with(')') {
        return format!("fn {}", rest);
    }

    // Find arrow for return type
    let arrow_pos = parts.iter().position(|&p| p == "->");
    let (param_parts, ret_part) = match arrow_pos {
        Some(pos) => (&parts[1..pos], Some(&parts[pos+1..])),
        None => (&parts[1..], None),
    };

    let params = param_parts.join(", ");
    match ret_part {
        Some(rt) if !rt.is_empty() => format!("Function {}({}) -> {}", name, params, rt.join(" ")),
        _ => format!("Function {}({})", name, params),
    }
}

/// Handle print/println with auto-string.
/// Parses the raw text after print/println using a state machine:
///   - Everything up to a `$` is string literal (preserving spaces)
///   - `$identifier` is a variable reference
///   - Use `'...'` for explicit strings (also preserved)
///
/// Examples:
///   print Hello World     → print("Hello World")
///   print $var            → print(var)
///   print Hello $var      → print("Hello " + var)
///   print 'Hello' $var    → print("Hello " + var)
///   println               → println()
fn process_print(line: &str) -> String {
    let (is_ln, after_raw) = if let Some(rest) = line.strip_prefix("println") {
        (true, rest)
    } else if let Some(rest) = line.strip_prefix("print ") {
        (false, rest)
    } else if line == "print" {
        (false, "")
    } else {
        (false, "")
    };

    let after = after_raw.trim();
                    let fn_name = if is_ln { "PrintLine" } else { "Print" };
                    if after.starts_with('(') {
                        return format!("{}{}", fn_name, after);
    }

    if after.is_empty() {
        return format!("{}()", fn_name);
    }

    // State machine: parse the raw text after print/println
    // States: normal, in_string, in_var
    enum State { Normal, InString, InVar }
    let mut state = State::Normal;
    let mut parts: Vec<String> = Vec::new();
    let mut buf = String::new();
    let mut in_sq = false;  // inside single-quoted string

    for c in after.chars() {
        match state {
            State::Normal => {
                if c == '$' {
                    // $ marks the start of a variable reference
                    // Flush any accumulated string
                    if !buf.is_empty() {
                        parts.push(format!("\"{}\"", buf));
                        buf.clear();
                    }
                    state = State::InVar;
                } else if c == '\'' {
                    // Start of single-quoted string
                    if !buf.is_empty() {
                        parts.push(format!("\"{}\"", buf));
                        buf.clear();
                    }
                    in_sq = true;
                    state = State::InString;
                } else if c.is_whitespace() {
                    if !buf.is_empty() {
                        buf.push(' ');
                    }
                    // Skip leading/trailing whitespace
                } else if c == '+' || c == '-' || c == '*' || c == '/' {
                    // Operator — flush string and emit operator
                    if !buf.is_empty() {
                        parts.push(format!("\"{}\"", buf));
                        buf.clear();
                    }
                    let op = String::from(c);
                    parts.push(op);
                } else {
                    state = State::InString;
                    buf.push(c);
                }
            }
            State::InString => {
                if c == '\'' && in_sq {
                    // Close single-quoted string
                    in_sq = false;
                    parts.push(format!("\"{}\"", buf));
                    buf.clear();
                    state = State::Normal;
                } else if c == '$' && !in_sq {
                    // $ while building string — end string, start var
                    parts.push(format!("\"{}\"", buf));
                    buf.clear();
                    state = State::InVar;
                } else {
                    buf.push(c);
                }
            }
            State::InVar => {
                if c.is_alphanumeric() || c == '_' {
                    buf.push(c);
                } else {
                    // End of variable name
                    parts.push(buf.clone());
                    buf.clear();
                    state = State::Normal;
                    if c == '\'' {
                        in_sq = true;
                        state = State::InString;
                    } else if !c.is_whitespace() {
                        buf.push(c);
                    }
                }
            }
        }
    }

    // Flush remaining buffer
    match state {
        State::InString => {
            if !buf.is_empty() {
                parts.push(format!("\"{}\"", buf));
            }
        }
        State::InVar => {
            parts.push(buf);
        }
        State::Normal => {
            if !buf.is_empty() {
                parts.push(format!("\"{}\"", buf));
            }
        }
    }

    if parts.is_empty() {
        return format!("{}()", fn_name);
    }

    if parts.len() == 1 && !parts[0].starts_with('"') {
        return format!("{}({})", fn_name, parts[0]);
    }
    if parts.len() == 1 {
        return format!("{}({})", fn_name, parts[0]);
    }

    // Multiple parts: generate a block with separate Print calls
    // (avoids string concatenation issues in C backend)
    let mut block = String::from("{\n");
    for (i, part) in parts.iter().enumerate() {
        let name = if is_ln && i == parts.len() - 1 { "PrintLine" } else { "Print" };
        if part.starts_with('"') {
            block.push_str(&format!("        {}({});\n", name, part));
        } else {
            block.push_str(&format!("        {}({});\n", name, part));
        }
    }
    block.push_str("    }");
    block
}

fn looks_like_call(line: &str) -> bool {
    let t = line.trim();
    // Must have spaces (i.e., multiple tokens)
    if !t.contains(' ') { return false; }
    // Must not start with a keyword
    let keywords = ["var ", "let ", "const ", "return ", "if ", "while ", "loop ",
                     "for ", "else", "fn ", "entity ", "system ", "actor ", "view ", "state "];
    if keywords.iter().any(|kw| t.starts_with(kw)) { return false; }
    // Must not be an assignment
    if t.contains(" = ") { return false; }
    // First token must look like an identifier
    let first = t.split_whitespace().next().unwrap_or("");
    if first.is_empty() { return false; }
    if first.starts_with('"') || first.starts_with('\'') { return false; }
    if first.chars().next().map_or(true, |c| c.is_ascii_digit()) { return false; }
    true
}

fn process_implicit_call(line: &str) -> String {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 2 { return line.to_string(); }

    let fn_name = parts[0];
    let mut args = Vec::new();

    for &arg in &parts[1..] {
        if arg.starts_with('"') || arg.starts_with('\'') || arg.starts_with('$') {
            args.push(arg.to_string());
        } else if arg.parse::<i64>().is_ok() || arg.parse::<f64>().is_ok() {
            args.push(arg.to_string());
        } else if arg == "true" || arg == "false" {
            args.push(arg.to_string());
        } else if arg.contains(&['(', ')', '+', '-', '*', '/', '=', '<', '>', '!', '&', '|'][..]) {
            args.push(arg.to_string());
        } else {
            // Looks like a string argument
            args.push(format!("\"{}\"", arg));
        }
    }

    format!("{}({})", fn_name, args.join(", "))
}
