//! Metadata for source location information.
//!
//! This contains all metadata attributable to the entire parse chain.

use std::rc::Rc;
use std::path::PathBuf;

/// A source file being read.
#[derive(Clone)]
enum Source {
    /// Reading from standard input.
    Stdin,
    /// Reading from a text buffer.
    TextBuffer(Rc<String>),
    /// A particular source file.
    Source(Rc<PathBuf>),
}

impl Source {
    fn start(&self) -> impl Locator {
        SourceLocation {
            source: self.clone(),
            location: Default::default(),
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
#[derive(Clone, Copy, Default)]
pub struct Location {
    row: u64,
    column: u64,
    character: u64,
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

        Location { row, column, character }
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
#[derive(Clone, Copy, Default)]
pub struct Span {
    start: Location,
    end: Location,
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
#[derive(Clone)]
struct SourceLocation {
    source: Source,
    location: Location,
}

impl Locator for SourceLocation {
    fn location(&self) -> &Location {
        &self.location
    }

    fn location_after(&self, next: char) -> Self {
        SourceLocation {
            source: self.source.clone(),
            location: self.location.location_after(next),
        }
    }

    type Span = SourceSpan;

    fn span_to(&self, end: &Location) -> Self::Span {
        SourceSpan {
            source: self.source,
            span: self.location.span_to(end),
        }
    }
}

/// A span within a particular source file.
#[derive(Clone)]
struct SourceSpan {
    source: Source,
    span: Span,
}

impl Locator for SourceSpan {
    fn location(&self) -> &Location {
        self.span.location()
    }

    fn location_after(&self, next: char) -> Self {
        SourceSpan {
            source: self.source.clone(),
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

impl SpanLocator for SourceSpan {
    fn span(&self) -> &Span {
        &self.span
    }

    fn extended_span(&self, next: char) -> Self {
        SourceSpan {
            source: self.source.clone(),
            span: self.span.extended_span(next),
        }
    }
}
