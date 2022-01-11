use cursive_table_view::TableViewItem;
use winstructs::timestamp::WinTimestamp;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeysColumn {
    Name,
    LastWritten
}

#[derive(Clone, Debug)]
pub struct KeysLine {
    name: String,
    timestamp: WinTimestamp,
}

impl KeysLine {
    pub fn new(name: &str, timestamp: &WinTimestamp) -> Self {
        Self {
            name: name.to_owned(),
            timestamp: timestamp.clone()
        }
    }
}

impl TableViewItem<KeysColumn> for KeysLine {
    fn to_column(&self, column: KeysColumn) -> String {
        match column {
            KeysColumn::Name => self.name.to_owned(),
            KeysColumn::LastWritten => self.timestamp.to_datetime().to_rfc3339()
        }
    }

    fn cmp(&self, other: &Self, column: KeysColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            KeysColumn::Name => self.name.cmp(&other.name),
            KeysColumn::LastWritten => self.timestamp.value().cmp(&other.timestamp.value())
        }
    }
}
