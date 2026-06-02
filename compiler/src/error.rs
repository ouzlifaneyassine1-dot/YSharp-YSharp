use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "error"),
            DiagnosticLevel::Warning => write!(f, "warning"),
            DiagnosticLevel::Note => write!(f, "note"),
            DiagnosticLevel::Help => write!(f, "help"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub file: SmolStr,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Span {
    pub fn new(
        file: SmolStr,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Span { file, start_line, start_col, end_line, end_col }
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
    pub labels: Vec<Label>,
    pub suggestions: Vec<String>,
    pub help: Option<String>,
}

#[derive(Debug)]
pub struct Diagnostics {
    pub diagnostics: RefCell<Vec<Diagnostic>>,
    pub source_map: RefCell<HashMap<SmolStr, String>>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics {
            diagnostics: RefCell::new(Vec::new()),
            source_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn add(&self, level: DiagnosticLevel, message: impl Into<String>, span: Option<Span>) {
        self.diagnostics.borrow_mut().push(Diagnostic {
            level,
            code: String::new(),
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
        });
    }

    pub fn add_rich(
        &self,
        level: DiagnosticLevel,
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> usize {
        let mut diags = self.diagnostics.borrow_mut();
        let idx = diags.len();
        diags.push(Diagnostic {
            level,
            code: code.into(),
            message: message.into(),
            span,
            labels: Vec::new(),
            suggestions: Vec::new(),
            help: None,
        });
        idx
    }

    pub fn emit(&self) {
        let diags = self.diagnostics.borrow();
        let source = self.source_map.borrow();
        let count = diags.len();

        for (i, diag) in diags.iter().enumerate() {
            let (tag, color_start) = match diag.level {
                DiagnosticLevel::Error => ("error", "\x1b[1;31m"),
                DiagnosticLevel::Warning => ("warning", "\x1b[1;33m"),
                DiagnosticLevel::Note => ("note", "\x1b[1;34m"),
                DiagnosticLevel::Help => ("help", "\x1b[1;32m"),
            };
            let color_end = "\x1b[0m";

            // Header: [E001] error: message
            let code_str = if diag.code.is_empty() {
                String::new()
            } else {
                format!("[{}] ", diag.code)
            };
            eprintln!("{}{}{}{}: {}", color_start, code_str, tag, color_end, diag.message);

            // Source span context
            if let Some(ref span) = diag.span {
                if let Some(src) = source.get(&span.file) {
                    let lines: Vec<&str> = src.lines().collect();
                    let ctx_lines = 2;

                    if span.start_line > 0 && span.start_line <= lines.len() {
                        // Context before
                        let ctx_start = if span.start_line > ctx_lines {
                            span.start_line - ctx_lines
                        } else {
                            1
                        };

                        for l in ctx_start..span.start_line {
                            if l <= lines.len() {
                                eprintln!(" {:>4} {}| {}", "", l, lines[l - 1]);
                            }
                        }

                        // Main line
                        eprintln!(" {:>4} {}| {}", "", span.start_line, lines[span.start_line - 1]);

                        // Underline / caret
                        let caret = if span.start_line == span.end_line && span.end_col > 0 {
                            let end = span.end_col.min(lines[span.start_line - 1].len() + 1);
                            let len = end.saturating_sub(span.start_col).max(1);
                            format!(
                                "{}{}",
                                " ".repeat(span.start_col.saturating_sub(1)),
                                "^".repeat(len)
                            )
                        } else {
                            format!(
                                "{}{}",
                                " ".repeat(span.start_col.saturating_sub(1)),
                                "^~~~"
                            )
                        };
                        eprintln!("      {}| {}", "", caret);

                        // Context after
                        for l in (span.start_line + 1)..(span.start_line + ctx_lines + 1) {
                            if l <= lines.len() {
                                eprintln!(" {:>4} {}| {}", "", l, lines[l - 1]);
                            }
                        }

                        // Labels
                        for label in &diag.labels {
                            let lbl_style = match label.style {
                                LabelStyle::Primary => " ",
                                LabelStyle::Secondary => ".",
                            };
                            eprintln!(
                                "      {} {} {}",
                                lbl_style,
                                if label.style == LabelStyle::Primary { "\x1b[1;31m" } else { "\x1b[1;34m" },
                                label.message,
                            );
                            eprintln!("      {} \x1b[0m", lbl_style);
                        }
                    }
                } else {
                    eprintln!(
                        "  {}--> {}:{}:{}",
                        "", span.file, span.start_line, span.start_col
                    );
                }
            }

            // Help text
            if let Some(ref help) = diag.help {
                eprintln!("  \x1b[1;32m=\x1b[0m {}: {}", "help", help);
            }

            // Suggestions
            for suggestion in &diag.suggestions {
                eprintln!("  \x1b[1;32m=\x1b[0m {}: {}", "suggestion", suggestion);
            }

            if i < count - 1 {
                eprintln!();
            }
        }
    }

    pub fn add_suggestion(&self, idx: usize, suggestion: impl Into<String>) {
        let mut diags = self.diagnostics.borrow_mut();
        if let Some(diag) = diags.get_mut(idx) {
            diag.suggestions.push(suggestion.into());
        }
    }

    pub fn set_help(&self, idx: usize, help: impl Into<String>) {
        let mut diags = self.diagnostics.borrow_mut();
        if let Some(diag) = diags.get_mut(idx) {
            diag.help = Some(help.into());
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .borrow()
            .iter()
            .any(|d| d.level == DiagnosticLevel::Error)
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}
