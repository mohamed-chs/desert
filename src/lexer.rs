use logos::Logos;
use std::collections::VecDeque;
use std::ops::Range;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("move")]
    Move,
    #[token("def")]
    Def,
    #[token("protocol")]
    Protocol,
    #[token("struct")]
    Struct,
    #[token("impl")]
    Impl,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("elif")]
    Elif,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    #[token("import")]
    Import,
    #[token("pyimport")]
    PyImport,
    #[token("match")]
    Match,
    #[token("as")]
    As,
    #[token("from")]
    From,
    #[token("enum")]
    Enum,

    #[token(":")]
    Colon,
    #[token("->")]
    Arrow,
    #[token("?")]
    Question,
    #[token("!!")]
    BangBang,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token("|")]
    Pipe,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token(">=")]
    Ge,
    #[token("<=")]
    Le,
    #[token("@")]
    At,
    #[token("$")]
    Dollar,
    #[token("&")]
    Ampersand,
    #[token("~")]
    Tilde,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("%")]
    Percent,
    #[token("/")]
    Slash,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    #[regex(r"\n[ \t]*")]
    NewlineWithIndent,

    #[regex(r"[ \t]+", logos::skip)]
    Whitespace,

    #[regex(r"#[^\n]*", logos::skip, allow_greedy = true)]
    Comment,

    Indent,
    Dedent,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
    indent_stack: Vec<usize>,
    pending_tokens: VecDeque<(Token, Range<usize>)>,
    eof_handled: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
            indent_stack: vec![0],
            pending_tokens: VecDeque::new(),
            eof_handled: false,
        }
    }

    pub fn span(&self) -> Range<usize> {
        self.inner.span()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Token, Range<usize>), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token_span) = self.pending_tokens.pop_front() {
            return Some(Ok(token_span));
        }

        match self.inner.next() {
            Some(Ok(Token::NewlineWithIndent)) => {
                let span = self.inner.span();
                let slice = self.inner.slice();
                let indent_str = &slice[1..];
                let mut current_indent = 0;
                for c in indent_str.chars() {
                    if c == ' ' {
                        current_indent += 1;
                    } else if c == '\t' {
                        current_indent += 4;
                    }
                }

                // Skip blank lines
                let mut temp_inner = self.inner.clone();
                match temp_inner.next() {
                    Some(Ok(Token::NewlineWithIndent)) | None => {
                        return self.next();
                    }
                    _ => {}
                }

                let last_indent = *self.indent_stack.last().unwrap();
                if current_indent > last_indent {
                    self.indent_stack.push(current_indent);
                    self.pending_tokens.push_back((Token::Indent, span.clone()));
                } else if current_indent < last_indent {
                    while current_indent < *self.indent_stack.last().unwrap() {
                        self.indent_stack.pop();
                        self.pending_tokens.push_back((Token::Dedent, span.clone()));
                    }
                    if current_indent != *self.indent_stack.last().unwrap() {
                        return Some(Err(()));
                    }
                }

                if let Some(t) = self.pending_tokens.pop_front() {
                    Some(Ok(t))
                } else {
                    self.next()
                }
            }
            Some(Ok(token)) => Some(Ok((token, self.inner.span()))),
            Some(Err(_)) => Some(Err(())),
            None => {
                if !self.eof_handled {
                    self.eof_handled = true;
                    let span = self.inner.span();
                    while self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        self.pending_tokens.push_back((Token::Dedent, span.clone()));
                    }
                    if let Some(token_span) = self.pending_tokens.pop_front() {
                        return Some(Ok(token_span));
                    }
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "let x = 10\ndef foo():\n    return x";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next().unwrap().unwrap().0, Token::Let);
        assert_eq!(
            lexer.next().unwrap().unwrap().0,
            Token::Ident("x".to_string())
        );
        assert_eq!(lexer.next().unwrap().unwrap().0, Token::Assign);
        assert_eq!(lexer.next().unwrap().unwrap().0, Token::Int(10));
        assert_eq!(lexer.next().unwrap().unwrap().0, Token::Def);
    }
}
