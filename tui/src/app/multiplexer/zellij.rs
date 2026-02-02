use std::{io, path::Path, process::Command};

use super::Multiplexer;

pub struct Zellij;

impl Multiplexer for Zellij {
    fn open(&self, path: &Path) -> io::Result<()> {
        let mut cmd = Command::new("zellij");
        cmd.args(["action", "edit"]).arg(path);
        cmd.spawn()?.wait()?;
        Ok(())
    }
}
