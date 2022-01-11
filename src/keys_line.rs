use cursive_table_view::TableViewItem;
use winstructs::timestamp::WinTimestamp;
use rwinreg::nk::NodeKey;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeysColumn {
    Name,
    LastWritten
}

#[derive(Clone, Debug)]
pub struct KeysLine {
    record: Rc<RefCell<NodeKey>>,
    name: String,
    timestamp: WinTimestamp,
}

impl KeysLine {
    pub fn from(nk: NodeKey) -> Self {
        let nk = Rc::new(RefCell::new(nk));
        let name = nk.borrow().key_name().to_owned();
        let timestamp = nk.borrow().get_last_written().clone();
        
        Self {
            record: nk,
            name: name,
            timestamp: timestamp
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
