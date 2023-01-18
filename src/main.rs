/* extern crate confy;
extern crate serde;
#[macro_use]
extern crate serde_derive; */

use clap::Parser;
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};
use std::error;
use std::io::{self, BufWriter, Read, StdoutLock, Write};
use std::path::{Path, PathBuf};

use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{
    Searcher, SearcherBuilder, Sink, SinkContext, SinkContextKind, SinkError, SinkFinish, SinkMatch,
};
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug, Parser)]
#[command(author, version, about = "Mine markdown for tasks", long_about = None)]
struct Cli {
    /// The pattern to look for
    #[arg(short, long)]
    pattern: Option<String>,
    /// The path to the file to read
    #[arg(default_values_os_t=vec![std::env::current_dir().unwrap_or(PathBuf::from("."))])]
    path_or_file: Vec<std::path::PathBuf>,
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

    info!("args: {:?}", args);
    info!("cfg: {:?}", cfg);

    let matcher = RegexMatcher::new(r"^\s*[*-] \[ \]")?;
    let mut file_searcher = SearcherBuilder::new()
        .before_context(50)
        .after_context(20)
        .build();
    let mut heading_searcher = SearcherBuilder::new().build();
    let mut tbuilder = TypesBuilder::new();
    tbuilder.add("markdown", "*.md");
    tbuilder.select("markdown");
    let tmatcher = tbuilder.build().unwrap();
    let mut walker_builder = WalkBuilder::new(args.path_or_file.first().unwrap());
    walker_builder
        .follow_links(true)
        .types(tmatcher)
        .standard_filters(true);
    for file_or_path in args.path_or_file.iter().skip(1) {
        walker_builder.add(file_or_path);
    }
    let walker = walker_builder.build();

    for file_or_path in walker {
        let fp = file_or_path?;
        let path = fp.path();
        // println!("{:?}, {:?}", &fp, fp.path());
        if path.is_dir() {
            continue;
        }

        file_searcher.search_path(&matcher, fp.path(), TaskOutput::new(fp.path()))?;
        //println!("{:?}", result);
    }

    /* searcher.search_path()
    Searcher::new().search_slice(&matcher, SHERLOCK, UTF8(|lnum, line| {
        // We are guaranteed to find a match, so the unwrap is OK.
        let mymatch = matcher.find(line.as_bytes())?.unwrap();
        matches.push((lnum, line[mymatch].to_string()));
        Ok(true)
    }))?; */

    Ok(())
}

pub struct TaskOutput<'a> {
    pub file: &'a Path,
    handle: BufWriter<StdoutLock<'a>>,
    matches: Vec<String>,
    before: Vec<String>,
    after: Vec<String>,
}

impl<'a> TaskOutput<'a> {
    fn new(file: &'a Path) -> TaskOutput<'a> {
        let stdout = io::stdout(); // get the global stdout entity
        let handle = io::BufWriter::new(stdout.lock());
        TaskOutput {
            file,
            handle,
            matches: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
        }
    }
}

impl<'a> Sink for TaskOutput<'a> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        let matched = match std::str::from_utf8(mat.bytes()) {
            Ok(matched) => matched,
            Err(err) => return Err(io::Error::error_message(err)),
        };
        let line_number = match mat.line_number() {
            Some(line_number) => line_number,
            None => {
                let msg = "line numbers not enabled";
                return Err(io::Error::error_message(msg));
            }
        };
        self.matches.push(format!("{} #L{}", matched, line_number));
        write!(self.handle, "{:?}:{}: {}", self.file, line_number, &matched)?;
        Ok(true)
    }
    fn context(
        &mut self,
        _searcher: &Searcher,
        _context: &SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        let context = std::str::from_utf8(_context.bytes())
            .map_err(|e| io::Error::error_message("context isn't utf-8"))?;
        match _context.kind() {
            SinkContextKind::Before => {
                // Display only headers
                // write!(self.handle, "{}", context)?;
                Ok(true)
            }
            SinkContextKind::After => {
                // write!(self.handle, "{}", context)?;
                Ok(true)
            }
            SinkContextKind::Other => Ok(true),
        }
    }
    fn finish(&mut self, searcher: &Searcher, finish: &SinkFinish) -> Result<(), io::Error> {
        Ok(())
    }
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
