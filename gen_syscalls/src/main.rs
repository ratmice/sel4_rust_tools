use argh::FromArgs;
use lazy_static::lazy_static;
use minijinja as jinja;
use sel4_xml_types::syscalls::*;
use std::io::Write as _;
use std::{fs, io, path};
use thiserror::Error;
mod lang_c;

#[derive(Error, Debug)]
enum SyscallGenError {
    #[error("xml error: {0}")]
    XMLError(#[from] quick_xml::DeError),
    #[error("io error: {0}")]
    IOError(#[from] io::Error),
    #[error("MiniJinja Error: {0}")]
    Minijinja(#[from] jinja::Error),
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

fn map_api_neg_range(
    api: Vec<(Option<String>, Vec<String>)>,
) -> Vec<(String, Vec<(String, isize)>)> {
    let mut neg_range = 1..std::isize::MAX;
    api.iter()
        .map(|config| {
            (
                match &config.0 {
                    None => "".to_string(),
                    Some(x) => x.clone(),
                },
                {
                    let ret = config
                        .1
                        .iter()
                        .cloned()
                        .zip(neg_range.clone().map(|i| -i))
                        .collect::<Vec<(String, isize)>>();
                    neg_range.start += config.1.len() as isize;
                    ret
                },
            )
        })
        .collect::<Vec<_>>()
}

lazy_static! {
    static ref RE: regex::Regex = regex::Regex::new(r"[A-Z][A-Z]?[^A-Z]*").unwrap();
}

fn convert_to_assembler_format(_state: &jinja::State, s: String) -> Result<String, jinja::Error> {
    Ok(RE
        .find_iter(&s)
        .map(|s| s.as_str().to_uppercase())
        .collect::<Vec<String>>()
        .join("_"))
}

fn main() -> Result<(), SyscallGenError> {
    // open files/parse xml so the user gets relevant errors first.
    let args: Args = argh::from_env();
    let f = fs::File::open(args.xml)?;
    let reader = io::BufReader::new(f);
    let syscalls: Syscalls = quick_xml::de::from_reader(reader)?;
    let mut env = jinja::Environment::new();

    // go through all the built in templates so a broken one can't go unnoticed.
    {
        env.add_function("upper", convert_to_assembler_format);
        env.add_template("kernel_header", lang_c::KERNEL_HEADER_TEMPLATE)?;
        env.add_template("libsel4_header", lang_c::LIBSEL4_HEADER_TEMPLATE)?;
        let _ = env.get_template("kernel_header")?;
        let _ = env.get_template("libsel4_header")?;
    }

    let api = if args.mcs {
        syscalls.api_mcs
    } else {
        syscalls.api_master
    }
    .config
    .iter()
    .map(|config| {
        (
            config.condition.clone(),
            config
                .syscalls
                .iter()
                .map(|syscall| syscall.name.clone())
                .collect::<Vec<String>>(),
        )
    })
    .collect::<Vec<(Option<String>, Vec<String>)>>();

    if let Some(kernel_header) = args.kernel_header {
        let mut dest_file = fs::File::create(kernel_header)?;
        let template = env.get_template("kernel_header")?;
        let syscall_min: usize = api.iter().map(|api| api.1.len()).sum();
        let api_and_debug = {
            let mut api_debug = api.clone();
            api_debug.extend(syscalls.debug.config.iter().map(|config| {
                (
                    config.condition.clone(),
                    config
                        .syscalls
                        .iter()
                        .map(|syscall| syscall.name.clone())
                        .collect::<Vec<String>>(),
                )
            }));
            api_debug
        };
        let ctx = jinja::context!(
            assembler => map_api_neg_range(api.clone()),
            enum => map_api_neg_range(api_and_debug),
            syscall_min => -(syscall_min as isize),

        );
        dest_file.write_all(template.render(ctx)?.as_bytes())?;
    }

    if let Some(libsel4_header) = args.libsel4_header {
        let mut dest_file = fs::File::create(libsel4_header)?;
        let template = env.get_template("libsel4_header")?;
        let ctx = jinja::context!(
            assembler => map_api_neg_range(api),

        );
        dest_file.write_all(template.render(ctx)?.as_bytes())?;
    }

    Ok(())
}
