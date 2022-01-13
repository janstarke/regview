use cursive_table_view::TableViewItem;
use std::rc::Rc;
use std::cell::RefCell;
use nt_hive::{Hive, KeyNode};
use anyhow::Result;

use crate::mmap_byteslice::MmapByteSlice;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KeysColumn {
    Name,
    LastWritten
}

#[derive(Clone)]
pub struct KeysLine {
    name: String,
    is_parent: bool
}

impl KeysLine {
    pub fn from(nk: KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>) -> Result<Self> {
        let nk = Rc::new(RefCell::new(nk));
        let name = nk.borrow().name()?.to_string_lossy();
        //let timestamp = nk.borrow().get_last_written().clone();

        Ok(Self {
            //record: Some(nk),
            name: name,
            is_parent: false
        })
    }

    pub fn parent() -> Self {
        Self {
            //record: None,
            name: "[..]".to_owned(),
            //timestamp: WinTimestamp::from(0),
            is_parent: true
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
}

impl TableViewItem<KeysColumn> for KeysLine {
    fn to_column(&self, column: KeysColumn) -> String {
        match column {
            KeysColumn::Name => self.name.to_owned(),
            KeysColumn::LastWritten => { panic!("not supported"); }
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
                    KeysColumn::LastWritten => { panic!("not supported"); }
                }
            }
        }
    }
}
