/* extern crate confy;
extern crate serde;
#[macro_use]
extern crate serde_derive; */

use clap::Parser;
use log::info;
use serde_derive::{Deserialize, Serialize};
use std::io::{self, BufWriter, StdoutLock, Write};
use std::path::{Path, PathBuf};

// use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
// use grep::searcher::sinks::UTF8;
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

    // We need the matcher to get headings as well as tasks
    let matcher = RegexMatcher::new(r"^(#+\s|\s*[*-] \[ \] )")?;
    let mut file_searcher = SearcherBuilder::new()
        .before_context(0)
        .after_context(20)
        .build();
    let mut tbuilder = TypesBuilder::new();
    tbuilder.add("markdown", "*.md")?;
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

    Ok(())
}

fn count_leading_whitespace(s: &str) -> u8 {
    s.chars()
        .take_while(|ch| ch.is_whitespace())
        .map(|ch| match ch {
            ' ' => 1,
            '\t' => 4,
            _ => 0,
        })
        .sum()
}
fn count_leading_hashes(s: &str) -> u8 {
    s.chars().take_while(|ch| ch == &'#').map(|_| 1).sum()
}

fn filter_headers_to_parents(s: &str) -> String {
    let (result, _) = s
        .lines()
        .rev()
        .fold((String::new(), 0), |(s, lastlevel), line| {
            // Guard against garbage lines mixed in
            if !line.starts_with("#") {
                return (s, lastlevel);
            }
            let thislevel = count_leading_hashes(&line);

            if lastlevel == 0 || thislevel < lastlevel {
                // keeping the line
                // put line on front since we reveresed order above
                let new_s = format!("{}\n{}", line, s);
                (new_s, thislevel)
            } else {
                // discarding the line
                (s, lastlevel)
            }
        });
    result
}

pub struct TaskOutput<'a> {
    pub file: &'a Path,
    handle: BufWriter<StdoutLock<'a>>,
    last_match_indent: u8,
    process_after: bool,
    unprinted_headers: String,
    first_print: bool,
}

impl<'a> TaskOutput<'a> {
    fn new(file: &'a Path) -> TaskOutput<'a> {
        let stdout = io::stdout(); // get the global stdout entity
        let handle = io::BufWriter::new(stdout.lock());
        TaskOutput {
            file,
            handle,
            last_match_indent: 0,
            process_after: false,
            unprinted_headers: String::new(),
            first_print: true,
        }
    }
}

impl<'a> Sink for TaskOutput<'a> {
    type Error = io::Error;

    fn begin(&mut self, _searcher: &Searcher) -> Result<bool, Self::Error> {
        if let Some(filename) = self.file.file_name().and_then(|s| s.to_str()) {
            write!(self.handle, "\n\n--{}--\n", filename)?;
        }
        Ok(true)
    }

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, io::Error> {
        let matched = match std::str::from_utf8(mat.bytes()) {
            Ok(matched) => matched,
            Err(err) => return Err(io::Error::error_message(err)),
        };
        if matched.starts_with("#") {
            self.unprinted_headers.push_str(&matched);
        } else {
            self.last_match_indent = count_leading_whitespace(&matched);
            self.process_after = true;

            // only show relevant parent headers
            let unprinted = filter_headers_to_parents(&self.unprinted_headers);

            write!(
                self.handle,
                "{}{}{}\n",
                if self.first_print || unprinted.len() == 0 {
                    ""
                } else {
                    "\n" // leading extra space before header sections after first
                },
                unprinted,
                &matched.trim_end()
            )?;
            self.first_print = false;
            self.unprinted_headers = String::new();
        }
        Ok(true)
    }

    fn context(
        &mut self,
        _searcher: &Searcher,
        _context: &SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        let context = std::str::from_utf8(_context.bytes())
            .map_err(|e| io::Error::error_message(format!("context isn't utf-8 {}", e)))?;

        match _context.kind() {
            SinkContextKind::Before => {}
            SinkContextKind::After => {
                if self.process_after && count_leading_whitespace(&context) > self.last_match_indent
                {
                    // print as long as indent is greater
                    write!(self.handle, "{}", context)?;
                } else {
                    // stop printing if anything is equal or less on indent
                    self.process_after = false;
                }
            }
            SinkContextKind::Other => {} // no-op,
        }
        Ok(true)
    }

    fn finish(&mut self, _searcher: &Searcher, _finish: &SinkFinish) -> Result<(), io::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests_submodules {
    use assert_cmd::prelude::*; // Add methods on commands
    use assert_fs::prelude::*;
    use predicates::prelude::*; // Used for writing assertions
    use std::process::Command; // Run programs

    /* #[test]
    fn check_answer_validity() {
        assert_eq!(32 + 10, 42);
    } */
    #[test]
    fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("mdtask")?;

        cmd.arg("foobar").arg("test/file/doesnt/exist");
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("No such file or directory"));
        Ok(())
    }

    #[test]
    fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
        let file = assert_fs::NamedTempFile::new("sample.txt")?;
        file.write_str("A test\n\n* [ ] my task\n\t* additional info\nAnother test")?;

        let mut cmd = Command::cargo_bin("mdtask")?;
        cmd.arg(file.path());
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("my task\n\t* additional"));

        Ok(())
    }

    #[test]
    fn check_count_leading_whitespace() {
        assert_eq!(super::count_leading_whitespace("no leading space"), 0);
        assert_eq!(super::count_leading_whitespace(" one leading space"), 1);
        assert_eq!(super::count_leading_whitespace("\ttab leading space"), 4);
        assert_eq!(
            super::count_leading_whitespace("    \ttab and four leading spaces"),
            8
        );
    }

    #[test]
    fn check_count_leading_hashes() {
        assert_eq!(super::count_leading_hashes("nothing here"), 0);
        assert_eq!(super::count_leading_hashes("# Heading 1"), 1);
        assert_eq!(super::count_leading_hashes("### Heading 3"), 3);
        assert_eq!(
            super::count_leading_hashes("#-# Not a heading but count should be one"),
            1
        );
        assert_eq!(super::count_leading_hashes("##### Heading 5"), 5);
    }

    #[test]
    fn check_filter_headers_to_parents() {
        assert_eq!(
            super::filter_headers_to_parents(
                r#"# Top Level
## Ignore me
### Ignore me
## Sub heading two
### Ignore me
### Sub two two
#### Sub sub two three"#
            ),
            r#"# Top Level
## Sub heading two
### Sub two two
#### Sub sub two three
"#
        );
    }
}
