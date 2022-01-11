use anyhow::Result;
use cursive::Cursive;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::DummyView;
use cursive::{views::{LinearLayout, ResizedView, Panel, NamedView, ViewRef}, CursiveRunnable};
use cursive_table_view::{TableView};
use cursive::event::{EventTrigger, EventResult};
use std::{cell::RefCell};

use crate::registry_hive::RegistryHive;
use crate::keys_line::*;
use crate::values_line::*;

static NAME_KEYS_PANE: &str = "keys_pane";
static NAME_KEYS_TABLE: &str = "keys_table";

pub struct UIMain {
    siv: RefCell<CursiveRunnable>
}

impl UIMain {
    pub fn new(hive: RegistryHive) -> Self {
        let mut siv = cursive::default();  
        siv.add_global_callback('q', |s| s.quit());

        let mut keys_table = TableView::<KeysLine, KeysColumn>::new()
            .column(KeysColumn::Name, "Name", |c| {c})
            .column(KeysColumn::LastWritten, "Last written", |c| c.width(25))
            ;

        keys_table.set_on_submit(UIMain::on_submit);
       
        let table = NamedView::new(NAME_KEYS_TABLE, keys_table);

        let mut details_table = TableView::<ValuesLine, ValuesColumn>::new()
            .column(ValuesColumn::Name, "Name", |c| c.width_percent(100));

        let items = vec![
            ValuesLine::new("Details1"), 
            ValuesLine::new("Details2"),
            ValuesLine::new("Details3")];
        details_table.set_items(items);
        details_table.set_enabled(false);

        let root_view = LinearLayout::horizontal()
            .child(table.with_name("keys").full_screen())
            .child(DummyView)
            .child(details_table.with_name("details").full_screen());

        siv.add_layer(Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            root_view
        )).title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));

        siv.set_user_data(hive);
        Self {
            siv: RefCell::new(siv)
        }
    }

    fn on_submit(siv: &mut Cursive, row: usize, index: usize) {
        let mut keys_table: ViewRef<TableView::<KeysLine, KeysColumn>> = siv.find_name(NAME_KEYS_TABLE).unwrap();
        let new_items = match keys_table.borrow_item(index) {
            None => { return },
            Some(item) => {
                let hive: &mut RegistryHive = siv.user_data().unwrap();
                if item.is_parent() {
                    hive.parent_keys().unwrap()
                } else {
                    hive.child_keys(item.record()).unwrap()
                }
            }
        };
        keys_table.clear();
        keys_table.set_items(new_items);
    }

    pub fn run(&self) -> Result<()> {
        let items = {
            let mut siv = self.siv.borrow_mut();
            let hive: &mut RegistryHive = siv.user_data().unwrap();
            hive.current_keys()?
        };
        self.siv.borrow_mut().call_on_name(NAME_KEYS_TABLE, |v: &mut TableView<KeysLine, KeysColumn>| v.set_items(items));
        self.siv.borrow_mut().run();
        Ok(())

    }
}