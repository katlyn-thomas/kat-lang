use std::{str, collections::HashMap};
use anyhow::{Result, bail};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // user generated
    Ident(String),
    Lit(Literal),
    // keywords
    Let,
    Mut,
    Def,
    Struct,
    Enum,
    Object,
    If,
    Elif,
    Else,
    Match,
    True,
    False,
    // newlines/whitespace
    NewLine,
    Space,
    Tab,
    // surrounding chars
    LParen,
    RParen,
    LSquirly,
    RSquirly,
    LBrack,
    RBrack,
    SingleQuote,
    DoubleQuote,
    LAngle,
    RAngle,
    // symbols
    Comma,
    Dot,
    Pipe,
    Plus,
    Dash,
    Underscore,
    Equal,
    FSlash,
    BSlash,
    Colon,
    SemiColon,
    Bang,
    At,
    Octothorpe,
    Dollar,
    Percent,
    Caret,
    Ampersand,
    Asterisk,
    Question,
    Tilde,
    Grave,

    EOF,
}


#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Str(String),
    Num(f64),
}

#[derive(Clone, Debug, PartialEq)]
struct LexicalAnalysis {
    lines: Vec<usize>, // we will store the char location of every newline here
    tokens: HashMap<usize, Token> // each token is stored with the first char's location as its key
}

pub struct Lexer {
    position: usize,
    read_position: usize,
    ch: u8,
    input:Vec<u8>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lex = Self {
            position: 0,
            read_position: 0,
            ch: 0,
            input: input.into_bytes(),
        };
        lex.next_char();

        lex
    }

    fn next_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position]
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> (usize, Token) {
        let tok = match self.ch {
            b'{'  => Token::LSquirly,
            b'}'  => Token::RSquirly,
            b'('  => Token::LParen,
            b')'  => Token::RParen,
            b'['  => Token::LBrack,
            b']'  => Token::RBrack,
            b'<'  => Token::LAngle,
            b'>'  => Token::RAngle,
            b','  => Token::Comma,
            b'.'  => Token::Dot,
            b'?'  => Token::Question,
            b':'  => Token::Colon,
            b';'  => Token::SemiColon,
            b'!'  => Token::Bang,
            b'@'  => Token::At,
            b'#'  => Token::Octothorpe,
            b'$'  => Token::Dollar,
            b'%'  => Token::Percent,
            b'^'  => Token::Caret,
            b'&'  => Token::Ampersand,
            b'*'  => Token::Asterisk,
            b'-'  => Token::Dash,
            b'='  => Token::Equal,
            b'+'  => Token::Plus,
            b'|'  => Token::Pipe,
            b'\\' => Token::BSlash,
            b'/'  => Token::FSlash,
            b'~'  => Token::Tilde,
            b'`'  => Token::Grave,
            b'\t' => Token::Tab,
            b' '  => self.read_whitespace(),

            b'\'' | b'"' => Token::Lit(Literal::Str(self.read_string_literal().to_string())),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let ident = self.read_ident();
                match ident.as_str() {
                    "let"    => Token::Let,
                    "mut"    => Token::Mut,
                    "def"    => Token::Def,
                    "struct" => Token::Struct,
                    "enum"   => Token::Enum,
                    "object" => Token::Object,
                    "if"     => Token::If,
                    "elif"   => Token::Elif,
                    "else"   => Token::Else,
                    "match"  => Token::Match,
                    "true"   => Token::True,
                    "false"  => Token::False,
                    _ => Token::Ident(ident.to_string())
                }
            },
            b'0'..=b'9' => Token::Lit(Literal::Num(self.read_number_literal())),
            0 => Token::EOF,
            _ => Token::EOF,
        };

        self.next_char();
        (self.position, tok)
    }

    fn read_whitespace(&mut self) -> Token {
        if self.prev_match(b'\t') || self.prev_match(b'\n') {
            if let Ok(matched) = self.peek_match("   ") { // 4 spaces to a tab?
                if matched {                              // there's got to be a better way...
                    self.next_char();
                    self.next_char();
                    self.next_char();
                    Token::Tab
                }
                else { Token:: Space }
            } else {
                Token::EOF
            }
        } else {
            Token::Space
        }
    }

    fn read_number_literal(&mut self) -> f64 { 
        let pos = self.position;
        let mut decimal = false;
        loop {
            match self.ch {
                b'0'..=b'9' => self.next_char(),
                b'.' => if decimal { break } else {
                    decimal = true;
                    self.next_char();
                },
                _ => break,
            }
        }
        String::from_utf8_lossy(&self.input[pos..self.position])
            .to_string()
            .parse::<f64>()
            .unwrap()
    }

    fn read_string_literal(&mut self) -> String { 
        // TODO: escape quote \"
        let quote = self.ch; // store the current single/double quote
        self.next_char(); // advance to first char of string
        let pos = self.position;
        loop {
            if self.ch == quote { break }
            else { self.next_char() }
        }
        String::from_utf8_lossy(&self.input[pos..self.position]).to_string()
    }

    fn read_ident(&mut self) -> String {
        let pos = self.position;
        while self.ch.is_ascii_alphabetic() || self.ch == b'_' {
            self.next_char();
        }
        String::from_utf8_lossy(&self.input[pos..self.position]).to_string()
    }
    
    fn peek_match(&self, input: &str) -> Result<bool> {
        if input.chars().count() + self.read_position >= self.input.len() { bail!("EOF") };

        let mut forward = 1;
        for ch in input.chars() {
            if self.input[self.position + forward] != ch as u8 { return Ok(false) }
            else { forward += 1 }
        }
        Ok(true)
    }

    fn prev_match(&self, input: u8) -> bool {
        if self.input[self.position - 1] == input { true } else { false }
    }
}
