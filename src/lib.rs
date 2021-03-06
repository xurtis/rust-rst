//! reStructuredText parser.
//!
//! This parses reStructuredText as described in the [specification][].
//!
//! [specification]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html

extern crate url;

pub mod ast;
pub mod location;
mod tokens;

pub use self::tokens::{Token, TokenStream};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
