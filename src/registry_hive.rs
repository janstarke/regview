use std::borrow::BorrowMut;
use std::fs::File;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use anyhow::Result;

use rwinreg::hive::Hive;
use rwinreg::nk::NodeKey;
use rwinreg::vk::Data;

use crate::keys_line::KeysLine;

pub struct RegistryHive {
    hive_file: File,
    root: NodeKey,
    path: Vec<String>
}

impl RegistryHive {
    pub fn new(hive_file: File) -> Result<Self> {
        let mut hive = Hive::from_source(&hive_file)?;
        let mut root = hive.get_root_node()?;
        Ok(Self {
            hive_file: hive_file,
            root: root,
            path: Vec::new()
        })
    }

    pub fn current_keys(&mut self) -> Result<Vec<KeysLine>> {
        let mut keys = Vec::new();

        loop {
            let record = match self.root.get_next_key(&mut self.hive_file)? {
                None => { break; }
                Some(node) => node
            };
            keys.push(KeysLine::from(record));
        }
        Ok(keys)
    }
}