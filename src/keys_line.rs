use std::{rc::Rc, cell::RefCell};

use cursive_table_view::TableViewItem;
use nt_hive2::KeyNode;
use chrono::DateTime;
use chrono::Utc;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeysColumn {
    NodeType,
    Name,
    LastWritten
}

#[derive(Clone)]
pub struct KeysLine {
    //record: KeyNode,
    name: String,
    is_parent: bool,
    is_leaf_node: bool,
    last_written: DateTime<Utc>
}

impl KeysLine {
    pub fn from(nk: &Rc<RefCell<KeyNode>>) -> Self {
        let name = nk.borrow().name().to_owned();
        let last_written = *nk.borrow().timestamp();
        Self {
            //record: nk,
            name,
            is_parent: false,
            is_leaf_node: nk.borrow().subkey_count() == 0,
            last_written
        }
    }

    pub fn parent() -> Self {
        Self {
            //record: None,
            name: "[..]".to_owned(),
            //timestamp: WinTimestamp::from(0),
            is_parent: true,
            is_leaf_node: false,
            last_written: DateTime::<Utc>::MIN_UTC
        }
    }
    pub fn is_parent(&self) -> bool {
        self.is_parent
    }
/* 
    pub fn record(&self) -> Rc<RefCell<KeyNode>> {
        Rc::clone(&self.record.as_ref().unwrap())
    }
*/
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_leaf_node(&self) -> bool {
        self.is_leaf_node
    }
}

impl TableViewItem<KeysColumn> for KeysLine {
    fn to_column(&self, column: KeysColumn) -> String {
        match column {
            KeysColumn::NodeType => {
                if self.is_leaf_node {"".to_owned()}
                else if self.is_parent {
                    "^".to_owned()
                } else {
                    "v".to_owned()
                }
            }
            KeysColumn::Name => self.name.to_owned(),
            KeysColumn::LastWritten => {
                if self.last_written == DateTime::<Utc>::MIN_UTC {
                    "".to_owned()
                } else {
                    self.last_written.format("%F %T").to_string()
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
        } else if other.is_parent {
            std::cmp::Ordering::Greater
        } else {
            match column {
                KeysColumn::NodeType => self.is_leaf_node.cmp(&other.is_leaf_node),
                KeysColumn::Name => self.name.cmp(&other.name),
                KeysColumn::LastWritten => self.last_written.cmp(&other.last_written),
            }
        }
    }
}
