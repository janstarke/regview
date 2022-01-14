use cursive_table_view::TableViewItem;
use nt_hive::{Hive, KeyNode};
use anyhow::{Result, anyhow};

use crate::mmap_byteslice::MmapByteSlice;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeysColumn {
    NodeType,
    Name,
    //LastWritten
}

#[derive(Clone)]
pub struct KeysLine {
    name: String,
    is_parent: bool,
    is_leaf_node: bool
}

impl KeysLine {
    pub fn from(nk: KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>) -> Result<Self> {
        let name = nk.name()?.to_string_lossy();
        //let timestamp = nk.borrow().get_last_written().clone();
        let subkey_count = match nk.subkeys() {
            None => 0,
            Some(subkeys_result) => {
                match subkeys_result {
                    Err(why) => {return Err(anyhow!(why));}
                    Ok(subkeys) => subkeys.count()
                }
            }
        };
        Ok(Self {
            //record: Some(nk),
            name: name,
            is_parent: false,
            is_leaf_node: subkey_count == 0
        })
    }

    pub fn parent() -> Self {
        Self {
            //record: None,
            name: "[..]".to_owned(),
            //timestamp: WinTimestamp::from(0),
            is_parent: true,
            is_leaf_node: false
        }
    }
    pub fn is_parent(&self) -> bool {
        self.is_parent
    }
/*
    pub fn record(&self) -> Rc<RefCell<KeyNode<&'a Hive<&'a [u8]>, &'a [u8]>>> {
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
                else {
                    if self.is_parent {
                        "\u{2303}".to_owned()
                    } else {
                        "\u{2304}".to_owned()
                    }
                }
            }
            KeysColumn::Name => self.name.to_owned(),
            //KeysColumn::LastWritten => { panic!("not supported"); }
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
                    KeysColumn::NodeType => self.is_leaf_node.cmp(&other.is_leaf_node),
                    KeysColumn::Name => self.name.cmp(&other.name),
                    //KeysColumn::LastWritten => { panic!("not supported"); }
                }
            }
        }
    }
}
