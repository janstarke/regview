use std::fs::File;
use std::cell::{RefCell, Ref};
use std::ops::Add;
use std::rc::Rc;
use anyhow::Result;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;

use rwinreg::hive::Hive;
use rwinreg::nk::NodeKey;
use rwinreg::vk::Data;

use crate::keys_line::KeysLine;
use crate::values_line::ValuesLine;

pub struct RegistryHive {
    hive_file: File,
    root: Rc<RefCell<NodeKey>>,
    path: Vec<Rc<RefCell<NodeKey>>>,
    visible_path: Vec<String>,

    values_cache: HashMap<u64, Rc<Vec<ValuesLine>>>
}

impl RegistryHive {
    pub fn new(hive_file: File) -> Result<Self> {
        let mut hive = Hive::from_source(&hive_file)?;
        let mut root = hive.get_root_node()?;
        Ok(Self {
            hive_file: hive_file,
            root: Rc::new(RefCell::new(root)),
            path: Vec::new(),
            visible_path: Vec::new(),
            values_cache: HashMap::new(),
        })
    }

    pub fn path(&self) -> String {
        self.visible_path.join("\\")
    }

    pub fn current_keys(&mut self) -> Result<Vec<KeysLine>> {
        let mut keys = vec![KeysLine::parent()];

        loop {
            //                                              question mark above sbwe's head ---------+
            //                                                                                       |
            //                                                                                       v
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
                self.visible_path.pop();
            }
        };
        
        self.current_keys()
    }

    pub fn child_keys(&mut self, item: Rc<RefCell<NodeKey>>) -> Result<Vec<KeysLine>> {
        self.visible_path.push(self.root.borrow().key_name().to_owned());
        self.path.push(Rc::clone(&self.root));
        self.root = Rc::clone(&item);
        self.current_keys()
    }


    pub fn key_values(&mut self, record: Rc<RefCell<NodeKey>>) -> Result<Rc<Vec<ValuesLine>>> {
        let record_hash = Self::hash_of(record.borrow());
        match self.values_cache.get(&record_hash) {
            Some(values) => Ok(Rc::clone(values)),
            None => {
                let mut values = Vec::new();
                loop {
                    let value = match record.borrow_mut().get_next_value(&mut self.hive_file)? {
                        None => {break;}
                        Some(node) => node
                    };
        
                    values.push(ValuesLine::from(&value)?);
                }
                let values = Rc::new(values);
                self.values_cache.insert(record_hash, Rc::clone(&values));
                Ok(values)

            }
        }
    }

    fn hash_of<T>(v: Ref<T>) -> u64 where T: Hash{
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }
}