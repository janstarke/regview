use anyhow::Result;
use cursive::Cursive;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::DummyView;
use cursive::{views::{LinearLayout, ResizedView, Panel, TextView, ViewRef}, CursiveRunnable};
use cursive_table_view::{TableView};
use cursive::event::{EventTrigger, EventResult};
use std::{cell::RefCell};

use crate::registry_hive::RegistryHive;
use crate::keys_line::*;
use crate::values_line::*;

static NAME_KEYS_PANE: &str = "keys_pane";
static NAME_KEYS_TABLE: &str = "keys_table";
static NAME_VALUES_TABLE: &str = "values_table";
static NAME_PATH_LINE: &str = "path_line";

pub struct UIMain {
    siv: RefCell<CursiveRunnable>
}

impl UIMain {
    pub fn new(hive: RegistryHive) -> Self {
        let mut siv = cursive::default();  
        siv.add_global_callback('q', |s| s.quit());

        let mut keys_table = TableView::<KeysLine, KeysColumn>::new()
            .column(KeysColumn::Name, "Name", |c| {c})
            .column(KeysColumn::LastWritten, "Last written", |c| c.width(25));

        keys_table.set_on_submit(UIMain::on_submit);
        keys_table.set_on_select(UIMain::on_select);

        let mut details_table = TableView::<ValuesLine, ValuesColumn>::new()
            .column(ValuesColumn::Name, "Name", |c| {c})
            .column(ValuesColumn::Data, "Value", |c| {c.width(8)})
            .column(ValuesColumn::Type, "Datatype", |c| {c});

        details_table.set_enabled(false);

        let reg_view = LinearLayout::horizontal()
            .child(keys_table.with_name(NAME_KEYS_TABLE).full_screen())
            .child(DummyView)
            .child(details_table.with_name(NAME_VALUES_TABLE).full_screen());

        let root_view = LinearLayout::vertical()
            .child(TextView::new("").with_name(NAME_PATH_LINE))
            .child(DummyView)
            .child(reg_view);

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
        let hive: &mut RegistryHive = siv.user_data().unwrap();
        let new_items = match keys_table.borrow_item(index) {
            None => { return },
            Some(item) => {
                if item.is_parent() {
                    hive.parent_keys().unwrap()
                } else {
                    hive.child_keys(item.record()).unwrap()
                }
            }
        };
        keys_table.clear();
        keys_table.set_items(new_items);
        let path = hive.path();
        siv.call_on_name(NAME_PATH_LINE, |l: &mut TextView| l.set_content(path));
    }

    fn on_select(siv: &mut Cursive, row: usize, index: usize) {
        let mut keys_table: ViewRef<TableView::<KeysLine, KeysColumn>> = siv.find_name(NAME_KEYS_TABLE).unwrap();
        let new_items = match keys_table.borrow_item(index) {
            None => { return },
            Some(item) => {
                if item.is_parent() {
                    Vec::new()
                } else {
                    let hive: &mut RegistryHive = siv.user_data().unwrap();
                    hive.key_values(item.record()).unwrap()
                }
            }
        };

        let mut values_table: ViewRef<TableView::<ValuesLine, ValuesColumn>> = siv.find_name(NAME_VALUES_TABLE).unwrap();
        values_table.clear();
        values_table.set_items(new_items);
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