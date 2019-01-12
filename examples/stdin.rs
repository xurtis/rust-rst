//! A simple example that prcoesses stdin.

use std::io::{stdin, stdout, Write, BufReader, BufRead};
use std::error::Error;

use rst::TokenStream;

fn main() -> Result<(), Box<dyn Error>> {

    let tokens = TokenStream::from_reader(stdin());

    let mut stdout = stdout();
    let mut lines = BufReader::new(stdin()).lines();

    loop {
        write!(stdout, "> ").ok();
        stdout.flush();

        let line = match lines.next() {
            Some(line) => line?,
            None => break,
        };

        for token in TokenStream::from_reader(line.as_bytes()) {
            let token = token?;
            println!("{:?} -> {:?}", token, token.from_numeral());
        }
    }

    Ok(())
}
