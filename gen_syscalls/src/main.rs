use argh::FromArgs;
use sel4_xml_types::syscalls::*;
use std::{fs, io, path};
use thiserror::Error;

#[derive(Error, Debug)]
enum SyscallGenError {
    #[error("xml error: {0}")]
    XMLError(#[from] quick_xml::DeError),
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
}

/// gen_syscalls
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(option)]
    /// xml file path
    xml: path::PathBuf,
    /// kernel header output path
    #[argh(option)]
    kernel_header: Option<path::PathBuf>,
    #[argh(option)]
    /// libsel4 header output path
    libsel4_header: Option<path::PathBuf>,
    /// generate MCS api
    #[argh(switch, short = 'm')]
    mcs: bool,
}

fn main() -> Result<(), SyscallGenError> {
    let args: Args = argh::from_env();
    let f = fs::File::open(args.xml)?;
    let reader = io::BufReader::new(f);
    let _syscalls: Syscalls = quick_xml::de::from_reader(reader)?;
    Ok(())
}
