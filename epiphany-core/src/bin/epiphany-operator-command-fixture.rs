use anyhow::{Context, Result};
use epiphany_core::write_operator_command_interop_fixture;
use std::path::PathBuf;

fn main() -> Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: epiphany-operator-command-fixture <output-directory>")?;
    let manifest = write_operator_command_interop_fixture(&output)?;
    println!("{}", serde_json::to_string_pretty(&manifest)?);
    Ok(())
}
