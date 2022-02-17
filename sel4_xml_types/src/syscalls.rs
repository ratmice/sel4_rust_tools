use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Syscalls {
    pub api_master: Api,
    pub api_mcs: Api,
    pub debug: Api,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Api {
    pub config: Vec<Config>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub syscall: Vec<Syscall>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Syscall {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn quick_xml_works() -> Result<(), TestError> {
        let f = File::open("../../../kernel/libsel4/include/api/syscall.xml")?;
        let reader = BufReader::new(f);
        let r: Syscalls = quick_xml::de::from_reader(reader)?;
        println!("{:#?}", r);
        Ok(())
    }

    #[test]
    fn serde_xml_works() -> Result<(), TestError> {
        let f = File::open("../../../kernel/libsel4/include/api/syscall.xml")?;
        let reader = BufReader::new(f);
        let r: Syscalls = serde_xml_rs::de::from_reader(reader)?;
        println!("{:#?}", r);
        Ok(())
    }
}
