mod zellij;

pub use zellij::Zellij;

use std::{io, path::Path};

pub trait Multiplexer {
    fn open(&self, path: &Path) -> io::Result<()>;
}
