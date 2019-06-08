//! A simple example that prcoesses stdin.

use std::error::Error;
use std::io::stdin;

use rst::{
    location::{ReaderSource, TextSource},
    TokenStream,
};

fn main() -> Result<(), Box<dyn Error>> {
    let example = "An example: 'this' has some punctuation (special chars)";
    let mut source = TextSource::from_str("example", example);
    for token in TokenStream::try_new(&mut source)? {
        let (token, span) = token?;
        println!("{}: {:?} = {:?}", span, span.excerpt().unwrap(), token);
    }

    let mut source = ReaderSource::from_reader("stdin", stdin());
    for token in TokenStream::try_new(&mut source)? {
        let (token, span) = token?;
        println!("{}: {:?}", span, token);
    }

    Ok(())
}
