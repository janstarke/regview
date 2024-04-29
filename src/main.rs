
use anyhow::Result;

mod ui_main;
mod regviewapplication;
mod registry_hive;
mod keys_line;
mod values_line;
mod search_result;

use regviewapplication::*;

fn main() -> Result<()> {
    let app = RegViewApplication::new()?;
    app.run()
}
