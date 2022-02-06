#![cfg(test)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("generating invocations")]
    InvocationGen(#[from] crate::invocations::InvocationGenError),
    #[error("quick_xml deserialization failed")]
    Deserialization(#[from] quick_xml::DeError),
    #[error("xml-rs deserialization failed")]
    XMLRs(#[from] serde_xml_rs::Error),
    #[error("roxml parsing failed")]
    Roxmltree(#[from] roxmltree::Error),
    #[error("filesystem error")]
    Filesystem(#[from] std::io::Error),
}
