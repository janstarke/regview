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
    record: Option<Rc<RefCell<NodeKey>>>,
    name: String,
    timestamp: WinTimestamp,
    is_parent: bool
}

impl KeysLine {
    pub fn from(nk: NodeKey) -> Self {
        let nk = Rc::new(RefCell::new(nk));
        let name = nk.borrow().key_name().to_owned();
        let timestamp = nk.borrow().get_last_written().clone();

        Self {
            record: Some(nk),
            name: name,
            timestamp: timestamp,
            is_parent: false
        }
    }

    pub fn parent() -> Self {
        Self {
            record: None,
            name: "[..]".to_owned(),
            timestamp: WinTimestamp::from(0),
            is_parent: true
        }
    }
    pub fn is_parent(&self) -> bool {
        self.is_parent
    }

    pub fn record(&self) -> Rc<RefCell<NodeKey>> {
        Rc::clone(&self.record.as_ref().unwrap())
    }
}

impl TableViewItem<KeysColumn> for KeysLine {
    fn to_column(&self, column: KeysColumn) -> String {
        match column {
            KeysColumn::Name => self.name.to_owned(),
            KeysColumn::LastWritten => {
                if self.is_parent {
                    "".to_owned()
                } else {
                    self.timestamp.to_datetime().to_rfc3339()
                }
            }
        }
    }

    fn cmp(&self, other: &Self, column: KeysColumn) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        if self.is_parent {
            if other.is_parent {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Less
            }
        } else {
            if other.is_parent {
                std::cmp::Ordering::Greater
            } else {
                match column {
                    KeysColumn::Name => self.name.cmp(&other.name),
                    KeysColumn::LastWritten => self.timestamp.value().cmp(&other.timestamp.value())
                }
            }
        }
    }
}
