use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Syscalls {
    api_master: Api,
    api_mcs: Api,
    debug: Api,
}

#[derive(Serialize, Deserialize, Debug)]
struct Api {
    config: Vec<Config>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    syscall: Vec<Syscall>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Syscall {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn quick_xml_works() -> Result<(), TestError> {
        let f = File::open("../../../seL4test/kernel/libsel4/include/api/syscall.xml")?;
        let reader = BufReader::new(f);
        let r: Syscalls = quick_xml::de::from_reader(reader)?;
        println!("{:#?}", r);
        Ok(())
    }

    #[test]
    fn serde_xml_works() -> Result<(), TestError> {
        let f = File::open("../../../seL4test/kernel/libsel4/include/api/syscall.xml")?;
        let reader = BufReader::new(f);
        let r: Syscalls = serde_xml_rs::de::from_reader(reader)?;
        println!("{:#?}", r);
        Ok(())
    }
}
