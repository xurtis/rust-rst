//! Streaming tokeniser for reStructuredText.
//!
//! This takes a read stream and produces an iterator over the tokens from that stream.

use std::io::{BufReader, Read, BufRead};

use crate::error::*;

pub struct TokenStream<R> {
    buffer: Option<Token>,
    chars: Chars<R>,
}

impl<R: Read> TokenStream<R> {
    pub fn from_reader(reader: R) -> TokenStream<R> {
        TokenStream {
            buffer: None,
            chars: Chars::from_reader(reader),
        }
    }
}

impl<R: Read> Iterator for TokenStream<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (buffer, c) = match (self.buffer.take(), self.chars.next()) {
                (buffer, Some(Ok(c))) => (buffer, c),
                (buffer, Some(Err(error))) => {
                    self.buffer = buffer;
                    return Some(Err(error.into()));
                }
                (buffer, None) => {
                    return buffer.map(Ok);
                }
            };

            match (buffer, Token::parse_char(c)) {
                (Some(Token::Word(mut s)), None) => {
                    s.push(c);
                    self.buffer = Some(Token::Word(s));
                }
                (Some(s), None) => {
                    let mut word = String::new();
                    word.push(c);
                    self.buffer = Some(Token::Word(word));
                    break Some(Ok(s));
                }
                (Some(s), token @ Some(_)) => {
                    self.buffer = token;
                    break Some(Ok(s));
                }
                (None, token @ Some(_)) => {
                    break token.map(Ok);
                }
                (None, None) => {
                    let mut word = String::new();
                    word.push(c);
                    self.buffer = Some(Token::Word(word));
                }
            }
        }
    }
}

struct Chars<R> {
    next: usize,
    buffer: Vec<char>,
    source: BufReader<R>,
}

impl<R: Read> Chars<R> {
    fn from_reader(reader: R) -> Chars<R> {
        Chars {
            next: 0,
            buffer: Vec::new(),
            source: BufReader::new(reader),
        }
    }
}

impl<R: Read> Chars<R> {
    fn next_char(&mut self) -> Option<char> {
        if self.next < self.buffer.len() {
            let next = self.buffer[self.next];
            self.next += 1;
            Some(next)
        } else {
            None
        }
    }

    fn refill_buffer(&mut self) -> Result<()> {
        let mut line = String::new();
        match self.source.read_line(&mut line) {
            Ok(_) => {
                self.buffer = line.chars().collect();
                self.next = 0;
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

impl<R: Read> Iterator for Chars<R> {
    type Item = Result<char>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.next_char() {
            Some(Ok(c))
        } else {
            if let Err(err) = self.refill_buffer() {
                Some(Err(err.into()))
            } else {
                self.next_char().map(Ok)
            }
        }
    }
}

/*
 * Bullets: ``*`` ``+`` ``-``, ``•``, ``‣``, ``⁃``
 *
 * Delimiters: ``.``, ``:``
 *
 * Options: ``-``, ``=``, ``/``
 *
 * Escape: ``\``
 *
 * Doctest: ``>>>``
 *
 * Tables: ``=``, ``+``, ``-``
 *
 * Signfigance of whitespace and position.
 *
 * Indentation.
 *
 * Braces: ``[``, ``]``, ``|``, ``(``, ``)``
 *
 * RCS: ``$``
 *
 * Auto-symbol: ``*``
 *
 * Links: ``_``
 *
 * Adornments:
 *
 *  ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~
 *
 * Numbers, words
 */

/// A single token from the input stream.
#[derive(Debug)]
pub enum Token {
    // Whitespace
    Newline,
    Whitespace(char),

    // Bullets
    Bullet,
    TriangularBullet,
    HyphenBullet,

    // Any of the following punctuation can be used for adornment.

    // Punctuation
    Exclamation,
    DoubleQuote,
    SingleQuote,
    Hash,
    Dollar,
    Percent,
    Ampersand,
    Asterisk,
    Plus,
    Comma,
    Hyphen,
    Period,
    ForwardSlash,
    Colon,
    SemiColon,
    LessThan,
    Equal,
    GreaterThan,
    Question,
    At,
    BackSlash,
    Caret,
    Underscore,
    Backtick,
    Pipe,
    Tilde,

    // Parentheses
    OpenParen,
    CloseParen,
    // Square brackets
    OpenBracket,
    CloseBracket,
    // Curly braces
    OpenBrace,
    CloseBrace,

    // A word is a continuous run of characters that are neither whitespace nor
    // punctuation.
    Word(String),
}
use self::Token::*;

impl Token {
    fn parse_char(c: char) -> Option<Token> {
        let c = match c {
            '\n' => Newline,
            c if c.is_whitespace() => Whitespace(c),
            '•'  => Bullet,
            '‣'  => TriangularBullet,
            '⁃'  => HyphenBullet,
            '!'  => Exclamation,
            '"'  => DoubleQuote,
            '\'' => SingleQuote,
            '#'  => Hash,
            '$'  => Dollar,
            '%'  => Percent,
            '&'  => Ampersand,
            '*'  => Asterisk,
            '+'  => Plus,
            ','  => Comma,
            '-'  => Hyphen,
            '.'  => Period,
            '/'  => ForwardSlash,
            ':'  => Colon,
            ';'  => SemiColon,
            '<'  => LessThan,
            '='  => Equal,
            '>'  => GreaterThan,
            '?'  => Question,
            '@'  => At,
            '\\' => BackSlash,
            '^'  => Caret,
            '_'  => Underscore,
            '`'  => Backtick,
            '|'  => Pipe,
            '~'  => Tilde,
            '('  => OpenParen,
            ')'  => CloseParen,
            '['  => OpenBracket,
            ']'  => CloseBracket,
            '{'  => OpenBrace,
            '}'  => CloseBrace,
            _ => return None,
        };

        Some(c)
    }

    /// The token could represent a bullet.
    pub fn is_bullet(&self) -> bool {
        match self {
            Asterisk | Plus | Hyphen => true,
            Bullet | TriangularBullet | HyphenBullet => true,
            _ => false,
        }
    }

    /// The token could be an adornment.
    pub fn is_adornment(&self) -> bool {
        match self {
            Exclamation => true,
            DoubleQuote => true,
            Hash => true,
            Dollar => true,
            Percent => true,
            Ampersand => true,
            SingleQuote => true,
            OpenParen => true,
            CloseParen => true,
            Asterisk => true,
            Plus => true,
            Comma => true,
            Hyphen => true,
            Period => true,
            ForwardSlash => true,
            Colon => true,
            SemiColon => true,
            LessThan => true,
            Equal => true,
            GreaterThan => true,
            Question => true,
            Ampersand => true,
            OpenBracket => true,
            BackSlash => true,
            CloseBracket => true,
            Caret => true,
            Underscore => true,
            Backtick => true,
            OpenBracket => true,
            Pipe => true,
            CloseBracket => true,
            Tilde => true,
            _ => false,
        }
    }

    /// If the token is a matching brace for another character.
    pub fn closes(&self, open: &Token) -> bool {
        match (open, self) {
            (OpenParen, CloseParen) => true,
            (OpenBracket, CloseBracket) => true,
            (OpenBrace, CloseBrace) => true,
            _ => false,
        }
    }

    /// If the token could be part of a referece.
    pub fn reference_member(&self) -> bool {
        match self {
            Word(_) => true,
            Hyphen => true,
            Underscore => true,
            Period => true,
            Colon => true,
            Plus => true,
            _ => false,
        }
    }

    /// Is any kind of numeral.
    pub fn from_numeral(&self) -> Option<u64> {
        self.from_arabic_numeral()
            .or_else(|| self.from_latin_numeral())
            .or_else(|| self.from_roman_numeral())
    }

    /// Is an arabic numeral.
    pub fn from_arabic_numeral(&self) -> Option<u64> {
        if let Word(word) = self {
            match word.parse().ok()? {
                0 => None,
                number => Some(number),
            }
        } else {
            None
        }
    }

    /// Is a latin numeral.
    pub fn from_latin_numeral(&self) -> Option<u64> {
        if let Word(word) = self {
            if word.len() != 1 {
                return None;
            }

            let letter = word.chars().next()?;

            if letter.is_ascii_alphabetic() {
                let mut ascii = [0u8; 1];
                letter.to_ascii_uppercase().encode_utf8(&mut ascii);
                Some((ascii[0] - b'A' + 1) as u64)
            } else {
                None
            }
        } else {
            None
        }
    }

    const ROMAN_NUMERALS: &'static [(&'static str, usize, u64)] = &[
        ("MMMM", 2, 4000),
        ("M",    0, 1000),
        ("CM",   4, 900),
        ("D",    1, 500),
        ("CD",   2, 400),
        ("C",    0, 100),
        ("XC",   4, 90),
        ("L",    1, 50),
        ("XL",   2, 40),
        ("X",    0, 10),
        ("IX",   4, 9),
        ("V",    1, 5),
        ("IV",   2, 4),
        ("I",    0, 1),
    ];

    /// Is a roman numeral.
    pub fn from_roman_numeral(&self) -> Option<u64> {
        if let Word(word) = self {
            let word = if word.chars().all(|c| c.is_ascii_lowercase()) {
                word.to_uppercase()
            } else {
                word.to_owned()
            };

            let mut roman_numerals = &Self::ROMAN_NUMERALS[..];
            let mut word = &word[..];
            let mut total = 0;
            let mut last = vec![];

            while word.len() > 0 {
                let (index, (numeral, skip, value)) = roman_numerals.iter()
                    .enumerate()
                    .find(|(i, (n, _, _))| word.starts_with(n))?;

                if last.len() > 0 && last[0] == numeral {
                    if *skip == 0 {
                        last.push(numeral);
                        if last.len() > 3 {
                            return None;
                        }
                    } else {
                        return None;
                    }
                } else {
                    last = vec![numeral];
                }

                total += value;
                word = &word[numeral.len()..];

                if *skip == 0 {
                    roman_numerals = &roman_numerals[index..];
                } else {
                    roman_numerals = &roman_numerals[index + skip..];
                }
            }

            if total != 0 {
                Some(total)
            } else {
                None
            }
        } else {
            None
        }
    }
}
