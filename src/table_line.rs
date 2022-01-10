use cursive_table_view::TableViewItem;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
}

#[derive(Clone, Debug)]
pub struct TableLine {
    name: String,
}

impl TableLine {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl TableViewItem<BasicColumn> for TableLine {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_owned(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
        }
    }
}
