use std::fs::File;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use anyhow::{anyhow, Result};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;

use memmap::{Mmap, MmapOptions};
use nt_hive::{Hive, KeyNode};

use crate::keys_line::KeysLine;
use crate::values_line::ValuesLine;
use crate::mmap_byteslice::MmapByteSlice;

pub struct RegistryHive {
    hive_file: File,
    //mmap: Mmap,
    hive: Hive<MmapByteSlice>,
    path: Vec<String>,

    values_cache: HashMap<u64, Rc<Vec<ValuesLine>>>,
}

impl RegistryHive {
    pub fn new(hive_file: File) -> Result<Self> {
        let mmap = unsafe { MmapOptions::new().map(&hive_file)? };
        let slice = MmapByteSlice::new(mmap);
        let mut hive = Hive::without_validation(slice)?;
        let mut root_node = hive.root_key_node()?;
        Ok(Self {
            hive_file,
            //mmap,
            hive,
            path: vec![],
            values_cache: HashMap::new(),
        })
    }

    pub fn path(&self) -> String {
        self.path.join("\\")
    }

    pub fn current_keys(&self) -> Result<Vec<KeysLine>> {
        let mut keys = vec![KeysLine::parent()];
        let root = self.hive.root_key_node()?;
        let current_node = root.subpath(&self.path()).unwrap()?;

        if let Some(subkeys_result) = current_node.subkeys() {
            match subkeys_result {
                Err(why) => {return Err(anyhow!(why));}
                Ok(subkeys) => {
                    for skn in subkeys {
                        match skn {
                            Err(why) => {return Err(anyhow!(why));}
                            Ok(k) => {keys.push(KeysLine::from(k)?);}
                        }
                    }
                }
            }
        }
        Ok(keys)
    }

    pub fn leave(&mut self) -> Result<Vec<KeysLine>> {
        if ! self.path.is_empty() {
            self.path.pop();
        }
        
        self.current_keys()
    }

    pub fn enter(&mut self, item_name: &str) -> Result<Vec<KeysLine>> {
        self.path.push(item_name.to_owned());
        self.current_keys()
    }

    pub fn key_values(&self, record_name: &str) -> Result<Vec<ValuesLine>> {
        
        let mut value_list = Vec::new();
        let root = self.hive.root_key_node()?;
        let current_node = root.subpath(&self.path()).unwrap()?;
        if let Some(values_result) = current_node.values() {
            match values_result {
                Err(why) => {return Err(anyhow!(why));}
                Ok(values) => {
                    for value in values {
                        match value {
                            Err(why) => {return Err(anyhow!(why));}
                            Ok(value) => {
                                value_list.push(ValuesLine::from(&value)?);
                            }
                        }
                    }
                }
            }
        }
        Ok(value_list)
    }

    fn hash_of<T>(v: Ref<T>) -> u64 where T: Hash{
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }
}