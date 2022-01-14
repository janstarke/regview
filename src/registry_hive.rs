use std::fs::File;
use anyhow::{anyhow, Result};
use regex::Regex;

use memmap::MmapOptions;
use nt_hive::{Hive, KeyNode};

use crate::keys_line::KeysLine;
use crate::values_line::ValuesLine;
use crate::mmap_byteslice::MmapByteSlice;


pub enum SearchResult {
    None,
    KeyName(Vec<String>),
    ValueName(Vec<String>, String),
    ValueData(Vec<String>, String)
}

impl SearchResult {
    pub fn is_none(&self) -> bool {
        matches!(*self, SearchResult::None)
    }
    pub fn is_some(&self) -> bool {
        ! self.is_none()
    }
}
pub struct RegistryHive {
    hive: Hive<MmapByteSlice>,
    path: Vec<String>,
}

impl RegistryHive {
    pub fn new(hive_file: File) -> Result<Self> {
        let mmap = unsafe { MmapOptions::new().map(&hive_file)? };
        let slice = MmapByteSlice::new(mmap);
        let hive = Hive::without_validation(slice)?;
        Ok(Self {
            hive,
            path: vec![],
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

    pub fn select_path(&mut self, path: &Vec<String>) -> Result<Vec<KeysLine>> {
        self.path = path.clone();
        self.current_keys()
    }

    pub fn selected_node(&self) -> Option<String> {
        self.path.last().and_then(|s|Some(s.to_owned()))
    }

    pub fn key_values(&self, record_name: &str) -> Result<Vec<ValuesLine>> {
        
        let mut value_list = Vec::new();
        let root = self.hive.root_key_node()?;
        let current_node = root.subpath(&(self.path() + "\\" + record_name)).unwrap()?;
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

    pub fn find_regex(&mut self, search_regex: &str) -> Result<SearchResult> {
        let regex = Regex::new(search_regex)?;
        let root = self.hive.root_key_node()?;
        let current_node = root.subpath(&(self.path())).unwrap()?;
        
        let mut search_path = self.path.clone();
        let result = self.find_in_subkeys(&mut search_path, &current_node, &regex)?;
        if result.is_some() {
            return Ok(result);
        }
        self.find_in_siblings(&mut search_path, &current_node, &regex)
    }

    fn find_in_siblings(&self, current_path: &mut Vec<String>, current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>, search_regex: &Regex) -> Result<SearchResult> {
        let root = self.hive.root_key_node()?;
        match current_path.pop() {
            None => {
                // we already have the root node, which has no siblings
                return Ok(SearchResult::None);
            }

            Some(_) => {
                let parent_node = root.subpath(&current_path.join("\\")).unwrap()?;
                let mut found_current_node = false;
                
                if let Some(subkeys_result) = parent_node.subkeys() {
                    match subkeys_result {
                        Err(why) => {return Err(anyhow!(why));}
                        Ok(subkeys) => {
                            for subkey_result in subkeys {
                                match subkey_result {
                                    Err(why) => {return Err(anyhow!(why));}
                                    Ok(subkey) => {
                                        if found_current_node {
                                            let result = self.find_in_this_node(current_path, &subkey, search_regex)?;
                                            if result.is_some() {
                                                return Ok(result);
                                            }

                                            let result = self.find_in_subkeys(current_path, &subkey, search_regex)?;
                                            if result.is_some() {
                                                return Ok(result);
                                            }
                                        } else {
                                            if subkey.name()? == current_node.name()? {
                                                found_current_node = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return self.find_in_siblings(current_path, &parent_node, search_regex);
            }
        }
    }

    fn find_in_this_node(&self, current_path: &mut Vec<String>, current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>, search_regex: &Regex) -> Result<SearchResult> {
        let subkey_name = current_node.name()?.to_string_lossy();
        
        /*
         * key name matches
         */
        if search_regex.is_match(&subkey_name) {
            let mut current_path = current_path.clone();
            current_path.push(subkey_name);
            return Ok(SearchResult::KeyName(current_path));
        }


        /*
         * check attributes
         */
        current_path.push(subkey_name);
        let result = self.find_in_attributes(current_path, &current_node, search_regex)?;
        if result.is_some() {
            return Ok(result);
        }
        current_path.pop();
        Ok(SearchResult::None)
    }

    fn find_in_subkeys(&self, current_path: &mut Vec<String>, current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>, search_regex: &Regex) -> Result<SearchResult> {
        if let Some(subkeys_result) = current_node.subkeys() {
            match subkeys_result {
                Err(why) => {return Err(anyhow!(why));}
                Ok(subkeys) => {
                    for subkey_result in subkeys {
                        match subkey_result {
                            Err(why) => {return Err(anyhow!(why));}
                            Ok(subkey) => {
                                let result = self.find_in_this_node(current_path, &subkey, search_regex)?;
                                if result.is_some() {
                                    return Ok(result);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(SearchResult::None)
    }

    fn find_in_attributes(&self, current_path: &mut Vec<String>, current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>, search_regex: &Regex) -> Result<SearchResult> {
        if let Some(values_result) = current_node.values() {
            match values_result {
                Err(why) => {return Err(anyhow!(why));}
                Ok(values) => {
                    for value_result in values {
                        match value_result {
                            Err(why) => {return Err(anyhow!(why));}
                            Ok(value) => {
                                let value_name = value.name()?.to_string_lossy();

                                /*
                                 * value name matches
                                 */
                                if search_regex.is_match(&value_name) {
                                    let current_path = current_path.clone();
                                    return Ok(SearchResult::ValueName(current_path, value_name));
                                }

                                /*
                                 * value data matches
                                 */
                                if let Ok(data) = value.string_data() {
                                    if search_regex.is_match(&data) {
                                        let current_path = current_path.clone();
                                        return Ok(SearchResult::ValueData(current_path, value_name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(SearchResult::None)
    }
}