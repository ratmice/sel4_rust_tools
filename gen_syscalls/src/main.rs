use sel4_xml_types::syscalls::*;
use std::{fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
enum SyscallGenError {
    #[error("xml error: {0}")]
    XMLError(#[from] quick_xml::DeError),
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
}

fn main() -> Result<(), SyscallGenError> {
    let f = fs::File::open("../../../kernel/libsel4/include/api/syscall.xml")?;
    let reader = io::BufReader::new(f);
    let syscalls: Syscalls = quick_xml::de::from_reader(reader)?;
    Ok(())
}
