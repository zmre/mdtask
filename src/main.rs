extern crate confy;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use clap::Parser;
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};
use std::io::{self, Write};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    pattern: String,
    /// The path to the file to read
    path: std::path::PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyConfig {
    name: String,
    comfy: bool,
    foo: i64,
}
impl Default for MyConfig {
    fn default() -> Self {
        MyConfig {
            name: "Unknown".to_string(),
            comfy: true,
            foo: 42,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let cfg: MyConfig = confy::load("mdtask", None)?;

    // info!("args: {:#?}", args);

    let stdout = io::stdout(); // get the global stdout entity
    let mut handle = io::BufWriter::new(stdout.lock()); // optional: wrap that handle in a buffer
    writeln!(handle, "Hello, world!: {}", 42)?;

    println!("Hello, world!");
    Ok(())
}

#[cfg(test)]
mod tests_submodules {
    use assert_cmd::prelude::*; // Add methods on commands
    use assert_fs::prelude::*;
    use predicates::prelude::*; // Used for writing assertions
    use std::process::Command; // Run programs

    #[test]
    fn check_answer_validity() {
        assert_eq!(32 + 10, 42);
    }
    /* #[test]
    fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("mdtask")?;

        cmd.arg("foobar").arg("test/file/doesnt/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("could not read file"));

        Ok(())
    } */

    #[test]
    fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
        let file = assert_fs::NamedTempFile::new("sample.txt")?;
        file.write_str("A test\nActual content\nMore content\nAnother test")?;

        let mut cmd = Command::cargo_bin("mdtask")?;
        cmd.arg("test").arg(file.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("test\nAnother test"));

        Ok(())
    }
}
