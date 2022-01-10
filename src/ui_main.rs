use anyhow::Result;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::{views::{Dialog, TextView, LinearLayout, ResizedView, Panel}, CursiveRunnable, theme::BaseColor};
use cursive_table_view::{TableView, TableViewItem};
use std::{cell::RefCell, borrow::BorrowMut};
use cursive_aligned_view::Alignable;

static PARENT_DIRECTORY: &str = "[..]";

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
            BasicColumn::Name => {
                if self.name == other.name {
                    std::cmp::Ordering::Equal
                } else if self.name == PARENT_DIRECTORY {
                    std::cmp::Ordering::Less
                } else if other.name == PARENT_DIRECTORY {
                    std::cmp::Ordering::Greater
                } else {
                    self.name.cmp(&other.name)
                }
            }
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
            TableLine::new(PARENT_DIRECTORY), 
            TableLine::new("Test1"),
            TableLine::new("Test2")];
        table.set_items(items);

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
            .child(details_table.with_name("details").full_screen());

        siv.add_layer(Panel::new(ResizedView::new(
            SizeConstraint::Full,
            SizeConstraint::Full,
            root_view
        )).title(format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))));

        Self {
            siv: RefCell::new(siv)
        }
    }

    pub fn run(&self) -> Result<()> {
        self.siv.borrow_mut().run();
        Ok(())

    }
}