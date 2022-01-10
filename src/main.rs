use anyhow::Result;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode, ColorChoice};

mod ui_main;
mod regviewapplication;
mod registry_hive;
mod table_line;
use regviewapplication::*;


fn main() -> Result<()> {
    let _ = TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto);
    
    let app = RegViewApplication::new()?;
    app.run()
}
