use cursive_table_view::TableViewItem;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ValuesColumn {
    Name,
}

#[derive(Clone, Debug)]
pub struct ValuesLine {
    name: String,
}

impl ValuesLine {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl TableViewItem<ValuesColumn> for ValuesLine {
    fn to_column(&self, column: ValuesColumn) -> String {
        match column {
            ValuesColumn::Name => self.name.to_owned(),
        }
    }

    fn cmp(&self, other: &Self, column: ValuesColumn) -> std::cmp::Ordering
    where 
        Self: Sized,
    {
        match column {
            ValuesColumn::Name => self.name.cmp(&other.name),
        }
    }
}
