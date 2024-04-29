use clap::Parser;
use anyhow::Result;
use log::Level;
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

    #[clap(flatten)]
    pub(crate) verbose: clap_verbosity_flag::Verbosity,
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
    log_level: Option<Level>,
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

        let log_level = if cli.verbose.is_present() {
            if cli.verbose.is_silent() {
                None
            } else {
                cli.verbose.log_level()
            }
        } else {
            None
        };

        Ok(Self {
            hive: Rc::new(RefCell::new(RegistryHive::new(reg_file, cli.logfiles, cli.ignore_base_block)?)),
            log_level
        })

    }

    pub fn run(self) -> Result<()> {
        
        let mut ui = UIMain::new(self.hive, self.log_level);
        ui.run()
    }
}