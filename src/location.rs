//! Metadata for source location information.
//!
//! This contains all metadata attributable to the entire parse chain.

use std::borrow::Cow;
use std::fmt;
use std::io::{BufRead, BufReader, Read};
use std::str;

use failure::Error;

/// A character source.
pub trait Source {
    /// The iterator over characters in the source.
    type Chars: Iterator<Item = Result<char, Error>>;

    /// Get the name of the source.
    ///
    /// This is displayed when showing errors in the source.
    fn name(&self) -> Cow<str>;

    /// Get an excerpt from the source.
    fn excerpt(&self, span: Span) -> Cow<str>;

    /// Get an iterator over the characters in the source.
    fn chars(&mut self) -> Option<Self::Chars>;
}

#[derive(Debug)]
pub struct TextSource<'t> {
    name: String,
    buffer: &'t str,
}

impl<'t> TextSource<'t> {
    pub fn from_str(name: &str, text: &'t str) -> Self {
        TextSource {
            name: name.to_owned(),
            buffer: text,
        }
    }
}

impl<'t> Source for TextSource<'t> {
    type Chars = TextChars<'t>;

    fn name(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    fn excerpt(&self, span: Span) -> Cow<str> {
        unimplemented!()
    }

    fn chars(&mut self) -> Option<Self::Chars> {
        Some(TextChars(self.buffer.chars()))
    }
}

pub struct TextChars<'t>(str::Chars<'t>);

impl<'t> Iterator for TextChars<'t> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Ok)
    }
}

#[derive(Debug)]
pub struct ReaderSource<R> {
    name: String,
    reader: Option<R>,
}

impl<R: Read> ReaderSource<R> {
    pub fn from_reader(name: &str, reader: R) -> Self {
        ReaderSource {
            name: name.to_owned(),
            reader: Some(reader),
        }
    }
}

impl<R: Read> Source for ReaderSource<R> {
    type Chars = ReaderChars<R>;

    fn name(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    fn excerpt(&self, span: Span) -> Cow<str> {
        unimplemented!()
    }

    fn chars(&mut self) -> Option<Self::Chars> {
        self.reader.take().map(ReaderChars::from_reader)
    }
}

pub struct ReaderChars<R> {
    next: usize,
    buffer: Vec<char>,
    source: BufReader<R>,
}

impl<R: Read> ReaderChars<R> {
    fn from_reader(reader: R) -> ReaderChars<R> {
        ReaderChars {
            next: 0,
            buffer: Vec::new(),
            source: BufReader::new(reader),
        }
    }
}

impl<R: Read> ReaderChars<R> {
    fn next_char(&mut self) -> Option<char> {
        if self.next < self.buffer.len() {
            let next = self.buffer[self.next];
            self.next += 1;
            Some(next)
        } else {
            None
        }
    }

    fn refill_buffer(&mut self) -> Result<(), Error> {
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

impl<R: Read> Iterator for ReaderChars<R> {
    type Item = Result<char, Error>;

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

/// Locate a single position within the input.
pub trait Locator {
    /// Get the current location.
    fn location(&self) -> &Location;

    /// Get the subsequent location after seeing a particular character.
    fn location_after(&self, next: char) -> Self;

    /// The type of span produced when creating a reagion.
    type Span: SpanLocator;

    /// Create a span up to a given location.
    fn span_to(&self, end: &Location) -> Self::Span;
}

/// Locate a span within the input.
pub trait SpanLocator: Locator {
    /// Get the region in the file.
    fn span(&self) -> &Span;

    /// Extend the location to include the given character.
    fn extended_span(&self, next: char) -> Self;
}

/// A location within a stream of text.
#[derive(Debug, Clone, Copy, Default)]
pub struct Location {
    row: u64,
    column: u64,
    character: u64,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.row, self.column)
    }
}

impl Location {
    pub fn row(&self) -> u64 {
        self.row
    }

    pub fn column(&self) -> u64 {
        self.column
    }

    pub fn character(&self) -> u64 {
        self.character
    }
}

impl Locator for Location {
    fn location(&self) -> &Location {
        self
    }

    fn location_after(&self, next: char) -> Self {
        let (row, column) = match next {
            '\n' => (self.row + 1, 0),
            _ => (self.row, self.column + 1),
        };
        let character = self.character + 1;

        Location {
            row,
            column,
            character,
        }
    }

    type Span = Span;

    fn span_to(&self, end: &Location) -> Self::Span {
        Span {
            start: *self,
            end: *end,
        }
    }
}

/// A span between two locations within a stream of text.
///
/// Inclusive of the start and non-inclusive of the end.
#[derive(Debug, Clone, Copy, Default)]
pub struct Span {
    start: Location,
    end: Location,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl Span {
    pub fn start(&self) -> &Location {
        &self.start
    }

    pub fn end(&self) -> &Location {
        &self.end
    }
}

impl Locator for Span {
    fn location(&self) -> &Location {
        &self.start
    }

    fn location_after(&self, next: char) -> Self {
        Span {
            start: self.end,
            end: self.end.location_after(next),
        }
    }

    type Span = Self;

    fn span_to(&self, end: &Location) -> Self::Span {
        Span {
            start: self.start,
            end: *end,
        }
    }
}

impl SpanLocator for Span {
    fn span(&self) -> &Span {
        self
    }

    fn extended_span(&self, next: char) -> Self {
        Span {
            start: self.start,
            end: self.end.location_after(next),
        }
    }
}

/// A location within a particular source file.
#[derive(Debug, Copy)]
pub struct SourceLocation<'s, S> {
    source: &'s S,
    location: Location,
}

impl<'s, S: Source> fmt::Display for SourceLocation<'s, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.source.name(), self.location)
    }
}

impl<'s, S: Source> SourceLocation<'s, S> {
    pub fn source_start(source: &'s S) -> Self {
        SourceLocation {
            source,
            location: Default::default(),
        }
    }
}

impl<'s, S> Clone for SourceLocation<'s, S> {
    fn clone(&self) -> Self {
        SourceLocation {
            source: self.source,
            location: self.location.clone(),
        }
    }
}

impl<'s, S> Locator for SourceLocation<'s, S> {
    fn location(&self) -> &Location {
        &self.location
    }

    fn location_after(&self, next: char) -> Self {
        SourceLocation {
            source: self.source,
            location: self.location.location_after(next),
        }
    }

    type Span = SourceSpan<'s, S>;

    fn span_to(&self, end: &Location) -> Self::Span {
        SourceSpan {
            source: self.source,
            span: self.location.span_to(end),
        }
    }
}

/// A span within a particular source file.
#[derive(Debug, Copy)]
pub struct SourceSpan<'s, S> {
    source: &'s S,
    span: Span,
}

impl<'s, S: Source> fmt::Display for SourceSpan<'s, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.source.name(), self.span)
    }
}

impl<'s, S> Clone for SourceSpan<'s, S> {
    fn clone(&self) -> Self {
        SourceSpan {
            source: self.source,
            span: self.span.clone(),
        }
    }
}

impl<'s, S> Locator for SourceSpan<'s, S> {
    fn location(&self) -> &Location {
        self.span.location()
    }

    fn location_after(&self, next: char) -> Self {
        SourceSpan {
            source: self.source,
            span: self.span.location_after(next),
        }
    }

    type Span = Self;

    fn span_to(&self, end: &Location) -> Self::Span {
        SourceSpan {
            source: self.source,
            span: self.span.span_to(end),
        }
    }
}

impl<'s, S> SpanLocator for SourceSpan<'s, S> {
    fn span(&self) -> &Span {
        &self.span
    }

    fn extended_span(&self, next: char) -> Self {
        SourceSpan {
            source: self.source,
            span: self.span.extended_span(next),
        }
    }
}
