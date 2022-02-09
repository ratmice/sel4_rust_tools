#![allow(unused_imports, unused_variables, dead_code)]
use sel4_xml_types::*;
use minijinja as jinja;
use argh::FromArgs;
use std::path::PathBuf;
use std::fs::File as _;
use std::io::Write as _;
use std::{fs, io};
use core::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error<'a> {
    #[error("Unknown language {0}")]
    UnknownLanguage(&'a str)
}

/// gen_syscalls
#[derive(FromArgs, Debug)]
struct TopArgs {
    /// rust | c       default: [rust]
    #[argh(option, from_str_fn(language_arg))]
    lang: Language,
    /// header template 
    #[argh(option)]
    header: PathBuf,
    /// arch template
    #[argh(option)]
    arch: PathBuf,
    /// xml files... 
    #[argh(option)]
    xml: Vec<PathBuf>,
    /// output file.
    #[argh(option, short = 'o')]
    out: PathBuf
}

#[derive(Debug)]
enum Language {
  C,
  Rust
}

#[allow(clippy::if_same_then_else)]
fn language_arg(s: &str) -> Result<Language, String> {
     match s.to_lowercase() {
         lang if lang == "rust" => {
             Ok(Language::Rust)
         }
         lang if lang == "c" => {
             Ok(Language::C)
         }
         dont_know => Err(format!("Unrecognized language '{}'", dont_know)),
     }
     
}

fn main() -> Result<(), std::io::Error> {
    let args: TopArgs = argh::from_env();
    let mut out_file = std::fs::File::create(args.out)?;
    out_file.write(b"const _: () = ();\n")?;
    Ok(())
}
