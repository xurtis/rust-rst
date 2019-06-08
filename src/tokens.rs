//! Streaming tokeniser for reStructuredText.
//!
//! This takes a read stream and produces an iterator over the tokens from that stream.

use crate::location::{Locator, Source, SourceLocation, SourceSpan, SpanLocator};

use failure::{format_err, Error};

pub struct TokenStream<'s, S: Source> {
    buffer: Option<(Token, SourceSpan<'s, S>)>,
    chars: Chars<'s, S>,
}

impl<'s, S: Source + 's> TokenStream<'s, S> {
    pub fn try_new(source: &'s mut S) -> Result<TokenStream<'s, S>, Error> {
        let stream = TokenStream {
            buffer: None,
            chars: Chars::try_from_source(source)?,
        };

        Ok(stream)
    }
}

impl<'s, S: Source> Iterator for TokenStream<'s, S> {
    type Item = Result<(Token, SourceSpan<'s, S>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (buffer, c, location) = match (self.buffer.take(), self.chars.next()) {
                (buffer, Some(Ok((c, loc)))) => (buffer, c, loc),
                (_, Some(Err(error))) => {
                    return Some(Err(error.into()));
                }
                (buffer, None) => {
                    return buffer.map(Ok);
                }
            };

            let next_location = location.location_after(c);
            let char_span = location.span_to(next_location.location());

            match (buffer, Token::parse_char(c)) {
                (Some((Token::Word(mut s), span)), None) => {
                    s.push(c);
                    self.buffer = Some((Token::Word(s), span.extended_span(c)));
                }
                (Some(t), None) => {
                    let mut word = String::new();
                    word.push(c);
                    self.buffer = Some((Token::Word(word), char_span));
                    break Some(Ok(t));
                }
                (Some(s), Some(token)) => {
                    self.buffer = Some((token, char_span));
                    break Some(Ok(s));
                }
                (None, Some(token)) => {
                    break Some(Ok((token, char_span)));
                }
                (None, None) => {
                    let mut word = String::new();
                    word.push(c);
                    self.buffer = Some((Token::Word(word), char_span));
                }
            }
        }
    }
}

/// A stream of characters.
pub struct Chars<'s, S: Source> {
    chars: S::Chars,
    location: SourceLocation<'s, S>,
}

impl<'s, S: Source> Chars<'s, S> {
    /// Open the standard input.
    fn try_from_source(source: &'s mut S) -> Result<Self, Error> {
        let chars = source
            .chars()
            .ok_or(format_err!("Couldn't read chars from {}", source.name()))?;

        let location = SourceLocation::source_start(source);

        Ok(Chars { chars, location })
    }
}

impl<'s, S: Source> Iterator for Chars<'s, S> {
    type Item = Result<(char, SourceLocation<'s, S>), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let location = self.location.clone();
        match self.chars.next()? {
            Ok(c) => {
                self.location = location.location_after(c);
                Some(Ok((c, location)))
            }
            Err(err) => Some(Err(err)),
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
    HyphenBullet,
    TriangularBullet,

    // Any of the following punctuation can be used for adornment.

    // Punctuation
    Ampersand,
    Asterisk,
    At,
    BackSlash,
    Backtick,
    Caret,
    Colon,
    Comma,
    Dollar,
    DoubleQuote,
    Equal,
    Exclamation,
    ForwardSlash,
    GreaterThan,
    Hash,
    Hyphen,
    LessThan,
    Percent,
    Period,
    Pipe,
    Plus,
    Question,
    SemiColon,
    SingleQuote,
    Tilde,
    Underscore,

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
use Token::*;

impl Token {
    fn parse_char(c: char) -> Option<Token> {
        let c = match c {
            '\n' => Newline,
            c if c.is_whitespace() => Whitespace(c),
            '•' => Bullet,
            '‣' => TriangularBullet,
            '⁃' => HyphenBullet,
            '!' => Exclamation,
            '"' => DoubleQuote,
            '\'' => SingleQuote,
            '#' => Hash,
            '$' => Dollar,
            '%' => Percent,
            '&' => Ampersand,
            '*' => Asterisk,
            '+' => Plus,
            ',' => Comma,
            '-' => Hyphen,
            '.' => Period,
            '/' => ForwardSlash,
            ':' => Colon,
            ';' => SemiColon,
            '<' => LessThan,
            '=' => Equal,
            '>' => GreaterThan,
            '?' => Question,
            '@' => At,
            '\\' => BackSlash,
            '^' => Caret,
            '_' => Underscore,
            '`' => Backtick,
            '|' => Pipe,
            '~' => Tilde,
            '(' => OpenParen,
            ')' => CloseParen,
            '[' => OpenBracket,
            ']' => CloseBracket,
            '{' => OpenBrace,
            '}' => CloseBrace,
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
            Ampersand => true,
            Asterisk => true,
            BackSlash => true,
            Backtick => true,
            Caret => true,
            CloseBracket => true,
            CloseParen => true,
            Colon => true,
            Comma => true,
            Dollar => true,
            DoubleQuote => true,
            Equal => true,
            Exclamation => true,
            ForwardSlash => true,
            GreaterThan => true,
            Hash => true,
            Hyphen => true,
            LessThan => true,
            OpenBracket => true,
            OpenParen => true,
            Percent => true,
            Period => true,
            Pipe => true,
            Plus => true,
            Question => true,
            SemiColon => true,
            SingleQuote => true,
            Tilde => true,
            Underscore => true,
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
        ("M", 0, 1000),
        ("CM", 4, 900),
        ("D", 1, 500),
        ("CD", 2, 400),
        ("C", 0, 100),
        ("XC", 4, 90),
        ("L", 1, 50),
        ("XL", 2, 40),
        ("X", 0, 10),
        ("IX", 4, 9),
        ("V", 1, 5),
        ("IV", 2, 4),
        ("I", 0, 1),
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
                let (index, (numeral, skip, value)) = roman_numerals
                    .iter()
                    .enumerate()
                    .find(|(_, (n, _, _))| word.starts_with(n))?;

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
