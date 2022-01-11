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
    root: Rc<RefCell<NodeKey>>,
    path: Vec<Rc<RefCell<NodeKey>>>
}

impl RegistryHive {
    pub fn new(hive_file: File) -> Result<Self> {
        let mut hive = Hive::from_source(&hive_file)?;
        let mut root = hive.get_root_node()?;
        Ok(Self {
            hive_file: hive_file,
            root: Rc::new(RefCell::new(root)),
            path: Vec::new()
        })
    }

    pub fn current_keys(&mut self) -> Result<Vec<KeysLine>> {
        let mut keys = vec![KeysLine::parent()];

        loop {
            let record = match self.root.borrow_mut().get_next_key(&mut self.hive_file)? {
                None => { break; }
                Some(node) => node
            };
            keys.push(KeysLine::from(record));
        }
        Ok(keys)
    }

    pub fn parent_keys(&mut self) -> Result<Vec<KeysLine>> {
        match self.path.pop() {
            None => (),
            Some(r) => {
                self.root = r;
            }
        };
        
        self.current_keys()
    }

    pub fn child_keys(&mut self, item: Rc<RefCell<NodeKey>>) -> Result<Vec<KeysLine>> {
        self.path.push(Rc::clone(&self.root));
        self.root = Rc::clone(&item);
        self.current_keys()
    }
}