use anyhow::Result;

mod ui_main;
mod regviewapplication;
mod registry_hive;
mod keys_line;
mod values_line;
mod mmap_byteslice;

use regviewapplication::*;

fn main() -> Result<()> {
    cursive::logger::init();
    let app: RegViewApplication = RegViewApplication::new()?;
    app.run()
}
