//! Streaming tokeniser for reStructuredText.
//!
//! This takes a read stream and produces an iterator over the tokens from that stream.

use std::io::Read;

///// A single token from the input stream.
