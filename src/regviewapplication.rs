use clap::{App, Arg};
use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use crate::ui_main::*;
use crate::registry_hive::*;

pub struct RegViewApplication {
    hive: Rc<RefCell<RegistryHive>>,
}

impl RegViewApplication {
    pub fn new() -> Result<Self> {
        Self::parse_options()
    }

    pub fn run(self) -> Result<()> {
        let mut ui = UIMain::new(self.hive);
        ui.run()
    }

    fn parse_options() -> Result<Self> {
        let app = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new("REG_FILE")
                    .help("path to registry hive file")
                    .required(true)
                    .multiple_occurrences(false)
                    .multiple_values(false)
                    .takes_value(true),
            );
        
        let matches = app.get_matches();
        let filename = matches.value_of("REG_FILE").expect("missing hive filename");
        let fp = PathBuf::from(&filename);
        let reg_file = if ! (fp.exists() && fp.is_file()) {
            return Err(anyhow::Error::msg(format!("File {} does not exist", &filename)));
        } else {
            File::open(fp)?
        };
        Ok(Self {
            hive: Rc::new(RefCell::new(RegistryHive::new(reg_file)?))
        })
    }
}