use std::fs::File;
use anyhow::{anyhow, Result};
use regex::Regex;
use thiserror::Error;

use memmap::MmapOptions;
use nt_hive::{Hive, KeyNode, KeyValueData, KeyValueDataType};

use crate::keys_line::KeysLine;
use crate::values_line::ValuesLine;
use crate::mmap_byteslice::MmapByteSlice;
use crate::search_result::SearchResult;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("No results found.")]
    NoResult,

    #[error("Too many results found.")]
    TooManyResults
}

pub struct RegistryHive {
    hive: Hive<MmapByteSlice>,
    path: Vec<String>,
}

impl RegistryHive {
    pub fn new(hive_file: File, omit_validation: bool) -> Result<Self> {
        let mmap = unsafe { MmapOptions::new().map(&hive_file)? };
        let slice = MmapByteSlice::new(mmap);
        let hive = if omit_validation{
            Hive::without_validation(slice)?
        } else {
            Hive::new(slice)?
        };
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
        let current_node = 
        match root.subpath(&self.path()) {
            None => {return Err(anyhow!("current path is invalid: '{}'", self.path()));}
            Some(node_result) => {match node_result {
                Err(why) => { return Err(anyhow!(why)); }
                Ok(node) => node
            }}
        };

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

    pub fn find_regex(&mut self, search_regex: &str) -> Result<Vec<SearchResult>> {
        let regex = Regex::new(search_regex)?;
        let root = self.hive.root_key_node()?;
        let current_node = root.subpath(&(self.path())).unwrap()?;
        let mut search_result = Vec::new();
        let mut current_path = Vec::new();
        
        self.find_here_and_below(&mut current_path, &current_node, &regex, &mut search_result)?;
        if search_result.is_empty() {
            return Err(anyhow!(SearchError::NoResult));
        } else {
            return Ok(search_result);
        }
    }

    fn find_here_and_below(&self,
                            current_path: &mut Vec<String>,
                            current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>,
                            search_regex: &Regex,
                            search_result: &mut Vec<SearchResult>) -> Result<()> {
        self.find_in_this_node(current_path, current_node, search_regex, search_result)?;
        self.find_in_subkeys_of(current_path, &current_node, search_regex, search_result)?;
        Ok(())
    }

    fn find_in_this_node(&self,
                        current_path: &mut Vec<String>,
                        current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>,
                        search_regex: &Regex,
                        search_result: &mut Vec<SearchResult>) -> Result<()> {
        let subkey_name = current_node.name()?.to_string_lossy();
        
        /*
         * key name matches
         */
        if search_regex.is_match(&subkey_name) {
            let current_path = current_path.clone();
            search_result.push(SearchResult::KeyName(current_path));
        }


        /*
         * check attributes
         */
        self.find_in_attributes(current_path, &current_node, search_regex, search_result)
    }

    fn find_in_subkeys_of(&self,
                        current_path: &mut Vec<String>,
                        current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>,
                        search_regex: &Regex,
                        search_result: &mut Vec<SearchResult>) -> Result<()> {
        if let Some(subkeys_result) = current_node.subkeys() {
            match subkeys_result {
                Err(why) => {return Err(anyhow!(why));}
                Ok(subkeys) => {
                    for subkey_result in subkeys {
                        match subkey_result {
                            Err(why) => {return Err(anyhow!(why));}
                            Ok(subkey) => {
                                let subkey_name = subkey.name()?.to_string_lossy();
                                current_path.push(subkey_name);
                                self.find_here_and_below(current_path, &subkey, search_regex, search_result)?;
                                current_path.pop();
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn find_in_attributes(&self,
                            current_path: &mut Vec<String>,
                            current_node: &KeyNode<&Hive<MmapByteSlice>, MmapByteSlice>,
                            search_regex: &Regex,
                            search_result: &mut Vec<SearchResult>) -> Result<()> {
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
                                let name_matches = 
                                if search_regex.is_match(&value_name) {
                                    true
                                } else {
                                    false
                                };

                                let mut matching_value = None;
                                match value.data_type()? {

                                    /*
                                     * value data matches (REG_SZ, REG_EXPAND_SZ)
                                     */
                                    KeyValueDataType::RegSZ | KeyValueDataType::RegExpandSZ => {
                                        let value = value.string_data()?;
                                            if search_regex.is_match(&value) {
                                                matching_value = Some(value);
                                            } 
                                    }

                                    /*
                                     * value data matches (REG_MULTI_SZ)
                                     */
                                    KeyValueDataType::RegMultiSZ => {
                                        if let Ok(values) = value.multi_string_data() {
                                            for value in values {
                                                if search_regex.is_match(&value) {
                                                    matching_value = Some(value);
                                                }
                                            }
                                        }
                                    }

                                    /*
                                    * search in binary data
                                    */
                                    KeyValueDataType::RegBinary => {
                                        if let Ok(value) = value.data() {
                                            match value {
                                                KeyValueData::Small(bytes) => {
                                                    let value = String::from_utf8_lossy(bytes); 
                                                    if search_regex.is_match(&value) {
                                                        matching_value = Some(value.to_string());
                                                    }
                                                }
        
                                                KeyValueData::Big(slice) => {
                                                    for bytes_result in slice {
                                                        match bytes_result {
                                                            Err(why) => return Err(anyhow!(why)),
                                                            Ok(bytes) => {
                                                                let value = String::from_utf8_lossy(bytes); 
                                                                if search_regex.is_match(&value) {
                                                                    matching_value = Some(value.to_string());
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    _ => ()
                                }
                                if name_matches {
                                    match matching_value {
                                        Some(value) => {
                                            add_search_result(search_result, SearchResult::ValueNameAndData(current_path.clone(), value_name, value))?;
                                        }
                                        None => {
                                            add_search_result(search_result, SearchResult::ValueName(current_path.clone(), value_name))?;
                                        }
                                    }
                                } else {
                                    if let Some(value) = matching_value {
                                        add_search_result(search_result, SearchResult::ValueData(current_path.clone(), value_name, value))?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn add_search_result(target: &mut Vec<SearchResult>, search_result: SearchResult) -> Result<()> {
    if target.len() > 999 {
        return Err(anyhow!(SearchError::TooManyResults));
    }
    target.push(search_result);
    Ok(())
}