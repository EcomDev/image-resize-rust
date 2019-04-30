mod command_parser;

fn main() {
    // To do some nice stuff
}

#[derive(Debug, PartialEq)]
pub enum Event {
    ImageFound {
        path: String
    },
    ImageNotFound {
        path: String
    },
    ImageResizeComplete {
        target: String
    },
    ImageResizeFailed {
        target: String,
        reason: String,
    }
}
