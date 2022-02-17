// Derived from header_invocation_gen.py and licensed the same
// Translation into rust 2022 Matt Rice
//#
//# Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
//#
//# SPDX-License-Identifier: BSD-2-Clause or GPL-2.0-only

#![allow(unused_imports, unused_variables, dead_code)]
use argh::FromArgs;
use core::str::FromStr;
use minijinja as jinja;
use sel4_xml_types::invocations::*;
use std::fs::File as _;
use std::io::Read as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::{fs, io};
use thiserror::Error;

mod lang_c;
mod lang_rust;

#[derive(Error, Debug)]
enum Error<'a> {
    #[error("Unknown language {0}")]
    UnknownLanguage(&'a str),
    #[error("Io Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("MiniJinja Error: {0}")]
    Minijinja(#[from] jinja::Error),
    #[error("RoXmlTree Error: {0}")]
    RoXmlTree(#[from] roxmltree::Error),
    #[error("seL4_xml_types error: {0}")]
    Sel4XmlTypes(#[from] sel4_xml_types::invocations::InvocationGenError),
}

// argh isn't quite working out here:
// It wants to rewrite sel4_arch to --sel4-arch
// it requires a short opt otherwise it wants to read
// --sel4-arch true and complains about e.g. --sel4-arch --dest
// because it is expecting a boolean.
// investigate alternatives, or just do arg parsing by hand.
/// gen_syscalls
#[derive(FromArgs, Debug)]
struct TopArgs {
    /// rust | c       default: [rust]
    #[argh(option, from_str_fn(language_arg))]
    lang: Language,
    /// libsel4
    #[argh(switch, short='l')]
    libsel4: bool,
    /// arch
    #[argh(switch, short='a')]
    arch: bool,
    /// sel4_arch
    #[argh(switch, short='s')]
    sel4_arch: bool,
    /// xml file...
    #[argh(option)]
    xml: PathBuf,
    /// output file.
    #[argh(option)]
    dest: PathBuf,
}

#[derive(Debug)]
enum Language {
    C,
    Rust,
}

#[allow(clippy::if_same_then_else)]
fn language_arg(s: &str) -> Result<Language, String> {
    match s.to_lowercase() {
        lang if lang == "rust" => Ok(Language::Rust),
        lang if lang == "c" => Ok(Language::C),
        dont_know => Err(format!("Unrecognized language '{}'", dont_know)),
    }
}

fn main() -> Result<(), Error<'static>> {
    let args: TopArgs = argh::from_env();
    let mut dest_file = std::fs::File::create(args.dest)?;
    let mut env = jinja::Environment::new();

    // So we catch any template parsing errors early
    // add all of them to the environment whether they get used or not.
    {
        env.add_template("C_invocation", lang_c::INVOCATION_TEMPLATE)?;
        env.add_template(
            "C_sel4_arch_invocation",
            lang_c::SEL4_ARCH_INVOCATION_TEMPLATE,
        )?;
        env.add_template("C_arch_invocation", lang_c::ARCH_INVOCATION_TEMPLATE)?;
        env.add_template("Rust_invocation", lang_rust::INVOCATION_TEMPLATE)?;
        env.add_template(
            "Rust_sel4_arch_invocation",
            lang_rust::SEL4_ARCH_INVOCATION_TEMPLATE,
        )?;
        env.add_template("Rust_arch_invocation", lang_rust::ARCH_INVOCATION_TEMPLATE)?;        
        let _ = env.get_template("C_invocation")?;
        let _ = env.get_template("C_sel4_arch_invocation")?;
        let _ = env.get_template("C_arch_invocation")?;
    
        let _ = env.get_template("Rust_invocation")?;
        let _ = env.get_template("Rust_sel4_arch_invocation")?;
        let _ = env.get_template("Rust_arch_invocation")?;
    }

    let xml_in = std::fs::File::open(args.xml)?;
    let mut reader = std::io::BufReader::new(xml_in);
    let mut s = String::new();
    let _len = reader.read_to_string(&mut s)?;
    let xml_tree = roxmltree::Document::parse(&s)?;
    let api: Api = Api::try_from(xml_tree)?;
    let header_title = if args.libsel4 { "LIBSEL4" } else { "API" };
    let lang = if let Language::C = args.lang {
        "C"
    } else {
        "Rust"
    };

    let mut invocation_list = Vec::new();
    for child in api.children {
        if let ApiElement::Interface { methods, .. } = child {
            for Method { id, condition, .. } in methods {
                invocation_list.push((id, condition));
            }
        }
    }

    let ctx = jinja::context!(
            libsel4 => args.libsel4,
            header_title => header_title,
            invocations => invocation_list,
    );

    let template = if args.arch {
        env.get_template(&format!("{lang}_invocation"))
    } else if args.libsel4 {
        env.get_template(&format!("{lang}_sel4_arch_invocation"))
    } else {
        env.get_template(&format!("{lang}_arch_invocation"))
    }?;
    dest_file.write_all(template.render(ctx)?.as_bytes())?;
    Ok(())
}
