use logos::Logos;
use smol_str::SmolStr;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // -- Keywords (29) --
    #[token("program")]
    Program,
    #[token("function")]
    Function,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("loop")]
    Loop,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("return")]
    Return,
    #[token("var")]
    Var,
    #[token("let")]
    Let,
    #[token("const")]
    Const,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("as")]
    As,
    #[token("entity")]
    Entity,
    #[token("component")]
    Component,
    #[token("system")]
    System,
    #[token("actor")]
    Actor,
    #[token("on")]
    On,
    #[token("state")]
    State,
    #[token("view")]
    View,
    #[token("model")]
    Model,
    #[token("tensor")]
    Tensor,
    #[token("differentiable")]
    Differentiable,
    #[token("async")]
    Async,
    #[token("await")]
    Await,

    // -- Symbols (26) --
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // -- Literals (4) --
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Int(i64),
    #[regex(r#""[^"]*""#, |lex| {
        let s = &lex.slice()[1 .. lex.slice().len() - 1];
        SmolStr::new(s)
    })]
    String(SmolStr),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| SmolStr::new(lex.slice()))]
    Ident(SmolStr),

    // -- Skippable --
    #[regex(r"[ \t\r\n]+", logos::skip)]
    Whitespace,
    #[regex(r"//[^\n]*|/\*[^*]*\*+(?:[^/*][^*]*\*+)*/", logos::skip)]
    Comment,

    // -- Error --
    Error,
}

impl Token {
    pub fn span(&self) -> (usize, usize) {
        (0, 0)
    }
}
