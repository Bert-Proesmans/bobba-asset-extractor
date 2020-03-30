use clap::{arg_enum, crate_authors, crate_description, crate_name, crate_version, Clap};
use humantime::Duration;
use std::path::PathBuf;
use std::time;

use crate::error;

pub(crate) struct CLI {
    pub zones: Vec<String>,
    pub version_pin: Option<String>,
    pub data_path: PathBuf,
    pub cache_time: time::Duration,
}

arg_enum! {
    #[derive(Debug)]
    enum TopLevelDomains {
        All,
        Com,
        De,
        Fi,
        Fr,
        It,
        Es,
        Nl,
        ComBr,
        ComTr
    }
}

#[derive(Clap)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!()
)]
struct CLIParse {
    #[clap(long, default_value = stringify!(ALL))]
    tld: TopLevelDomains,

    #[clap(long)]
    version_pin: Option<String>,

    #[clap(long, parse(from_os_str), default_value = "./data/")]
    data_path: PathBuf,

    #[clap(long, default_value = "10d")]
    cache_time: Duration,
}

fn parse_tld(tld: TopLevelDomains) -> Vec<String> {
    fn convert_tld(item: TopLevelDomains) -> String {
        match item {
            TopLevelDomains::All => panic!("Method shouldn't be called with variant All"),
            TopLevelDomains::ComBr => String::from("com.br"),
            TopLevelDomains::ComTr => String::from("com.tr"),
            other => format!("{}", other),
        }
    }

    match tld {
        TopLevelDomains::All => vec![
            convert_tld(TopLevelDomains::Com),
            convert_tld(TopLevelDomains::De),
            convert_tld(TopLevelDomains::Fi),
            convert_tld(TopLevelDomains::Fr),
            convert_tld(TopLevelDomains::It),
            convert_tld(TopLevelDomains::Es),
            convert_tld(TopLevelDomains::Nl),
            convert_tld(TopLevelDomains::ComBr),
            convert_tld(TopLevelDomains::ComTr),
        ],
        other => vec![convert_tld(other)],
    }
}

pub(crate) fn get_cli() -> Result<CLI, error::ExtractorError> {
    match CLIParse::try_parse() {
        Ok(parsed) => Ok(CLI {
            zones: parse_tld(parsed.tld),
            version_pin: parsed.version_pin,
            data_path: parsed.data_path,
            cache_time: parsed.cache_time.into(),
        }),
        Err(error) => Err(error::ExtractorError::Argument(error)),
    }
}
