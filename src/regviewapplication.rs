use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use crate::ui_main::*;
use crate::registry_hive::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path to registry hive file
    pub (crate) hive_file: String,

    /// transaction LOG file(s). This argument can be specified one or two times.
    #[clap(short('L'), long("log"))]
    #[arg(value_parser = validate_file)]
    logfiles: Vec<PathBuf>,

    /// ignore the base block (e.g. if it was encrypted by some ransomware)
    #[clap(short('I'), long)]
    ignore_base_block: bool,
}

fn validate_file(s: &str) -> Result<PathBuf, String> {
    let pb = PathBuf::from(s);
    if pb.is_file() && pb.exists() {
        Ok(pb)
    } else {
        Err(format!("unable to read file: '{s}'"))
    }
}

pub struct RegViewApplication {
    hive: Rc<RefCell<RegistryHive>>,
}

impl RegViewApplication {
    pub fn new() -> Result<Self> {
        let cli = Args::parse();

        let fp = PathBuf::from(&cli.hive_file);
        let reg_file = if ! (fp.exists() && fp.is_file()) {
            return Err(anyhow::Error::msg(format!("File {} does not exist", &cli.hive_file)));
        } else {
            File::open(fp)?
        };

        Ok(Self {
            hive: Rc::new(RefCell::new(RegistryHive::new(reg_file, cli.logfiles, cli.ignore_base_block)?))
        })

    }

    pub fn run(self) -> Result<()> {
        let mut ui = UIMain::new(self.hive);
        ui.run()
    }
}