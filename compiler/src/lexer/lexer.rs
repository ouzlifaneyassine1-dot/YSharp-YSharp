use logos::Logos;

use crate::error::{Diagnostic, DiagnosticLevel, Span};
use crate::lexer::token::Token;
use smol_str::SmolStr;

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
    source: &'a str,
    line_map: Vec<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let line_map = build_line_map(source);
        Lexer {
            inner: Token::lexer(source),
            source,
            line_map,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, (usize, usize))>, Diagnostic> {
        let mut tokens = Vec::new();

        while let Some(result) = self.inner.next() {
            let byte_range = self.inner.span();

            match result {
                Ok(token) => match token {
                    Token::Whitespace | Token::Comment => continue,
                    _ => tokens.push((token, (byte_range.start, byte_range.end))),
                },
                Err(()) => {
                    let bad = &self.source[byte_range.start..byte_range.end];
                    let ch = bad.chars().next().unwrap_or('\0');
                    let span = self.byte_to_span(
                        byte_range.start,
                        byte_range.end,
                        SmolStr::new(""),
                    );
                    return Err(Diagnostic {
                        level: DiagnosticLevel::Error,
                        code: String::new(),
                        message: format!("unexpected character '{}'", ch),
                        span: Some(span),
                        labels: vec![],
                        suggestions: vec![
                            "check for typos or unsupported syntax".into(),
                        ],
                        help: None,
                    });
                }
            }
        }

        Ok(tokens)
    }

    pub fn byte_to_span(
        &self,
        start_byte: usize,
        end_byte: usize,
        file: SmolStr,
    ) -> Span {
        let (start_line, start_col) =
            offset_to_line_col(&self.line_map, start_byte);
        let (end_line, end_col) =
            offset_to_line_col(&self.line_map, end_byte);
        Span::new(file, start_line, start_col, end_line, end_col)
    }
}

fn build_line_map(source: &str) -> Vec<usize> {
    let mut map = vec![0usize];
    for (i, c) in source.char_indices() {
        if c == '\n' {
            map.push(i + 1);
        }
    }
    map.push(source.len());
    map
}

fn offset_to_line_col(line_map: &[usize], offset: usize) -> (usize, usize) {
    match line_map.binary_search(&offset) {
        Ok(line) => (line + 1, 1),
        Err(line) => {
            let line_idx = line.saturating_sub(1);
            (line_idx + 1, offset - line_map[line_idx] + 1)
        }
    }
}
