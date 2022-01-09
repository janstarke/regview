use anyhow::Result;
use cursive::view::{Nameable, Resizable};
use cursive::{views::{Dialog, TextView}, CursiveRunnable, theme::BaseColor};
use cursive_table_view::{TableView, TableViewItem};
use std::{cell::RefCell, borrow::BorrowMut};
use cursive_aligned_view::Alignable;

pub struct UIMain {
    siv: RefCell<CursiveRunnable>
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Name
}

#[derive(Clone, Debug)]
struct TableLine {
    name: String
}

impl TableLine {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned()
        }
    }
}

impl TableViewItem<BasicColumn> for TableLine {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_owned()
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> std::cmp::Ordering
    where
            Self: Sized {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name)
        }
    }
}

impl UIMain {
    pub fn new() -> Self {
        let mut siv = cursive::default();  
        siv.add_global_callback('q', |s| s.quit());

        let mut table = TableView::<TableLine, BasicColumn>::new()
            .column(BasicColumn::Name, "Name", |c| c.width_percent(100));
        
        let mut items = vec![
            TableLine::new("[..]"), 
            TableLine::new("Test1"),
            TableLine::new("Test2")];
        table.set_items(items);
        siv.add_layer(Dialog::around(table.with_name("table").min_size((50, 20))).title("Table View"));

        Self {
            siv: RefCell::new(siv)
        }
    }

    pub fn run(&self) -> Result<()> {
        self.siv.borrow_mut().run();
        Ok(())

    }
}