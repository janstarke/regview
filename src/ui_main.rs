use anyhow::Result;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::DummyView;
use cursive::{views::{Dialog, TextView, LinearLayout, ResizedView, Panel, NamedView}, CursiveRunnable, theme::BaseColor};
use cursive_table_view::{TableView, TableViewItem};
use std::{cell::RefCell, borrow::BorrowMut};
use cursive_aligned_view::Alignable;

use crate::registry_hive::RegistryHive;
use crate::table_line::*;

static PARENT_DIRECTORY: &str = "[..]";

pub struct UIMain {
    siv: RefCell<CursiveRunnable>,
    hive: RefCell<RegistryHive>
}

impl UIMain {
    pub fn new(hive: RegistryHive) -> Self {
        let mut siv = cursive::default();  
        siv.add_global_callback('q', |s| s.quit());

        let mut table = NamedView::new("keys_table", TableView::<TableLine, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(100)));

        let mut details_table = TableView::<TableLine, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(100));

        let mut items = vec![
            TableLine::new("Details1"), 
            TableLine::new("Details2"),
            TableLine::new("Details3")];
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
        self.siv.borrow_mut().call_on_name("keys_table", |v: &mut TableView<TableLine, BasicColumn>| v.set_items(self.hive.borrow_mut().current_keys().unwrap()));
        self.siv.borrow_mut().run();
        Ok(())

    }
}