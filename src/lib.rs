//! reStructuredText parser.
//!
//! This parses reStructuredText as described in the [specification][].
//!
//! [specification]: http://docutils.sourceforge.net/docs/ref/rst/restructuredtext.html

pub mod ast;
pub mod tokens;
pub mod location;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
