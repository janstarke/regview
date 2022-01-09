use anyhow::Result;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ColorChoice};
use clap::{App, Arg};

struct RegViewApplication {}

impl RegViewApplication {
    pub fn new() -> Result<Self> {
        let mut me = Self {};
        me.parse_options()?;
        Ok(me)
    }

    pub fn run(&self) -> Result<()> {
        Ok(())
    }

    fn parse_options(&mut self) -> Result<()> {
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
        Ok(())
    }
}

fn main() -> Result<()> {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto);
    
    let app = RegViewApplication::new()?;
    app.run()
}
