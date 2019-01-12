
use error_chain::*;

error_chain! {
    types { Error, ErrorKind, ResultExt, Result; }

    foreign_links {
        Io(::std::io::Error);
    }

    errors {
    }
}
