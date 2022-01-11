use anyhow::Result;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::DummyView;
use cursive::{views::{LinearLayout, ResizedView, Panel, NamedView}, CursiveRunnable, theme::BaseColor};
use cursive_table_view::{TableView, TableViewItem};
use std::{cell::RefCell};

use crate::registry_hive::RegistryHive;
use crate::keys_line::*;
use crate::values_line::*;


pub struct UIMain {
    siv: RefCell<CursiveRunnable>,
    hive: RefCell<RegistryHive>
}

impl UIMain {
    pub fn new(hive: RegistryHive) -> Self {
        let mut siv = cursive::default();  
        siv.add_global_callback('q', |s| s.quit());

        let mut table = NamedView::new("keys_table", TableView::<KeysLine, KeysColumn>::new()
            .column(KeysColumn::Name, "Name", |c| {c})
            .column(KeysColumn::LastWritten, "Last written", |c| c.width(25))
        );

        let mut details_table = TableView::<ValuesLine, ValuesColumn>::new()
            .column(ValuesColumn::Name, "Name", |c| c.width_percent(100));

        let mut items = vec![
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

        Self {
            siv: RefCell::new(siv),
            hive: RefCell::new(hive)
        }
    }

    pub fn run(&self) -> Result<()> {
        self.siv.borrow_mut().call_on_name("keys_table", |v: &mut TableView<KeysLine, KeysColumn>| v.set_items(self.hive.borrow_mut().current_keys().unwrap()));
        self.siv.borrow_mut().run();
        Ok(())

    }
}