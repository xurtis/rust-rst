//! A simple example that prcoesses stdin.

use std::error::Error;
use std::io::stdin;

use rst::{location::ReaderSource, TokenStream};

fn main() -> Result<(), Box<dyn Error>> {
    let mut source = ReaderSource::from_reader("stdin", stdin());
    for token in TokenStream::try_new(&mut source)? {
        let (token, span) = token?;
        println!("{}: {:?}", span, token);
    }

    Ok(())
}
