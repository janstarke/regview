use anyhow::{anyhow, Result, bail};
use regex::Regex;
use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;
use std::convert::TryInto;
use thiserror::Error;

use nt_hive2::{Hive, KeyNode, RegistryValue, SubPath, HiveParseMode, CleanHive, ContainsHive};

use crate::keys_line::KeysLine;
use crate::search_result::SearchResult;
use crate::values_line::ValuesLine;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("No results found.")]
    NoResult,

    #[error("Too many results found.")]
    TooManyResults,
}

pub struct RegistryHive
{
    hive: RefCell<Hive<File, CleanHive>>,
    path: Vec<String>,
    root: Rc<RefCell<KeyNode>>,
}

impl RegistryHive {
    pub fn new(hive_file: File, mut logfiles: Vec<PathBuf>, ignore_base_block: bool) -> Result<Self> {

        let parse_mode = if ignore_base_block {
            let hive = Hive::new(&hive_file, HiveParseMode::Raw).unwrap();
            let offset = match hive.find_root_celloffset() {
                Some(offset) => offset,
                None => {
                    log::error!("scan found no root cell offset, aborting...");
                    std::process::exit(-1);
                }
            };
            //println!("found offset at {}", offset.0);
            HiveParseMode::Normal(offset)      
        } else {
            HiveParseMode::NormalWithBaseBlock
        };

        let hive = Hive::new(hive_file, parse_mode)?;

        let mut clean_hive = 
        match logfiles.len() {
            0 => {
                log::warn!("no log files provided, treating hive as if it was clean");
                hive.treat_hive_as_clean()
            }
            1 => {
                hive.with_transaction_log(File::open(logfiles.pop().unwrap())?.try_into()?)?
                .apply_logs()
            }
            2 => {
                hive.with_transaction_log(File::open(logfiles.pop().unwrap())?.try_into()?)?
                .with_transaction_log(File::open(logfiles.pop().unwrap())?.try_into()?)?
                .apply_logs()
            }
            _ => {
                bail!("more than two transaction log files are not supported")
            }
        };

        let root = clean_hive.root_key_node()?;
        Ok(Self { 
            hive:RefCell::new(clean_hive),
            path: vec![],
            root: Rc::new(RefCell::new(root))
        })
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn root(&self) -> &Rc<RefCell<KeyNode>> {
        &self.root
    }

    pub fn current_keys(&self) -> Result<Vec<KeysLine>> {
        let mut keys = vec![KeysLine::parent()];
        
        let current_node = if self.path().is_empty() {
            Rc::clone(self.root())
        } else {
            match self.root().borrow().subpath(self.path(), &mut self.hive.borrow_mut())? {
                None => return Err(anyhow!("current path is invalid: '{:?}'", self.path())),
                Some(node_result) => node_result
            }
        };

        for skn in current_node.borrow().subkeys(&mut self.hive.borrow_mut())?.iter() {
            keys.push(KeysLine::from(skn));
        }
        Ok(keys)
    }

    fn path_is_valid(&self) -> bool {
        self.is_path_valid(&self.path)
    }

    fn is_path_valid(&self, path: &Vec<String>) -> bool {
        let mut mut_hive = self.hive.borrow_mut();
        
        if path.is_empty() {
            return true;
        }

        if let Ok(root) = mut_hive.root_key_node() {
            match root.subpath(path, &mut mut_hive) {
                Ok(Some(_)) => return true,
                _ => return false
            }
        }
        false
    }

    pub fn leave(&mut self) -> Result<Vec<KeysLine>> {
        assert!(self.path_is_valid());
        if !self.path.is_empty() {
            self.path.pop();
        }

        let result = self.current_keys();
        assert!(self.path_is_valid());
        result
    }

    pub fn enter(&mut self, item_name: &str) -> Result<Vec<KeysLine>> {
        assert!(self.path_is_valid(), "invalid path: '{:?}'", self.path);

        self.path.push(item_name.to_owned());
        match self.current_keys() {
            Err(why) => {
                self.path.pop();
                assert!(self.path_is_valid());
                Err(why)
            }
            Ok(children) => {
                assert!(self.path_is_valid());
                Ok(children)
            }
        }
    }

    /// switches to a specified path and returns the child elements
    /// of that very path
    pub fn select_path(&mut self, path: &Vec<String>) -> Result<Vec<KeysLine>> {
        assert!(self.path_is_valid());

        if !self.is_path_valid(path) {
            return Err(anyhow!("invalid path specified: '{}'", path.join("\\")));
        }

        self.path = path.clone();

        match self.current_keys() {
            Err(why) => {
                self.path.pop();
                assert!(self.path_is_valid());
                Err(why)
            }
            Ok(children) => {
                assert!(self.path_is_valid());
                Ok(children)
            }
        }
    }

    pub fn selected_node(&self) -> Option<String> {
        self.path.last().map(|s| s.to_owned())
    }

    pub fn key_values(&self, record_name: &str) -> Result<Vec<ValuesLine>> {
        let mut value_list = Vec::new();
        let root = self.hive.borrow_mut().root_key_node()?;
        let mut path = self.path().clone();
        path.push(record_name.into());
        let path = path.join("\\");

        let current_node = match root.subpath(&path, &mut self.hive.borrow_mut())? {
            None => {
                log::warn!("there is no node with the name '{path}'");
                if self.path().is_empty() {
                    Rc::new(RefCell::new(root))
                } else {
                    return Err(anyhow!(
                        "the node with path '{}' contains no children",
                        &(self.path().join("\\") + "\\" + record_name)
                    ))
                }
            }
            Some(node) => node
        };

        for value in current_node.borrow().values() {
            value_list.push(ValuesLine::from(value)?);
        }
        Ok(value_list)
    }

    pub fn find_regex(&mut self, search_regex: &str) -> Result<Vec<SearchResult>> {
        let regex = Regex::new(search_regex)?;
        let root = self.hive.borrow_mut().root_key_node()?;
        let mut search_result = Vec::new();
        let mut current_path = Vec::new();

        self.find_here_and_below(&mut current_path, &root, &regex, &mut search_result)?;
        if search_result.is_empty() {
            Err(anyhow!(SearchError::NoResult))
        } else {
            Ok(search_result)
        }
    }

    fn find_here_and_below (
        &self,
        current_path: &mut Vec<String>,
        current_node: &KeyNode,
        search_regex: &Regex,
        search_result: &mut Vec<SearchResult>,
    ) -> Result<()> {
        self.find_in_this_node(current_path, current_node, search_regex, search_result)?;
        self.find_in_subkeys_of(current_path, current_node, search_regex, search_result)?;
        Ok(())
    }

    fn find_in_this_node(
        &self,
        current_path: &mut [String],
        current_node: &KeyNode,
        search_regex: &Regex,
        search_result: &mut Vec<SearchResult>,
    ) -> Result<()> {
        let subkey_name = current_node.name();

        /*
         * key name matches
         */
        if search_regex.is_match(subkey_name) {
            let current_path = Vec::from(&mut *current_path);
            search_result.push(SearchResult::KeyName(current_path));
        }

        /*
         * check attributes
         */
        self.find_in_attributes(current_path, current_node, search_regex, search_result)
    }

    fn find_in_subkeys_of(
        &self,
        current_path: &mut Vec<String>,
        current_node: &KeyNode,
        search_regex: &Regex,
        search_result: &mut Vec<SearchResult>,
    ) -> Result<()> {
        let subkeys: Vec<Rc<RefCell<KeyNode>>> = current_node.subkeys(&mut self.hive.borrow_mut())?
            .iter()
            .map(Rc::clone)
            .collect();
        for subkey in subkeys {
            let subkey_name = subkey.borrow().name().to_owned();
            current_path.push(subkey_name);
            self.find_here_and_below(
                current_path,
                &subkey.borrow(),
                search_regex,
                search_result,
            )?;
            current_path.pop();
        }
        Ok(())
    }

    fn find_in_attributes(
        &self,
        current_path: &mut[String],
        current_node: &KeyNode,
        search_regex: &Regex,
        search_result: &mut Vec<SearchResult>,
    ) -> Result<()> {
        for value in current_node.values() {

            /*
             * value name matches
             */
            let name_matches = search_regex.is_match(value.name());
            
            let matching_value =
            match value.value() {
                RegistryValue::RegSZ(val) |
                RegistryValue::RegExpandSZ(val) |
                RegistryValue::RegLink(val) |
                RegistryValue::RegResourceList(val) |
                RegistryValue::RegFullResourceDescriptor(val) |
                RegistryValue::RegResourceRequirementsList(val)
                    => search_regex.is_match(val).regview_then_some(val.to_owned()),

                RegistryValue::RegDWord(val)  |
                RegistryValue::RegDWordBigEndian(val) => {
                    let val = format!("0x{:08X}", val);
                    search_regex.is_match(&val).regview_then_some(val)
                }

                RegistryValue::RegQWord(val) => {
                    let val = format!("0x{:016X}", val);
                    search_regex.is_match(&val).regview_then_some(val)
                }

                RegistryValue::RegBinary(val) => {
                    let val = String::from_utf8_lossy(val).to_string();
                    search_regex.is_match(&val).regview_then_some(val)
                }
                RegistryValue::RegMultiSZ(val)
                    => val.iter().find(|s| search_regex.is_match(s)).map(|s| s.to_owned()),
                
                _ => None,
            };

            match (name_matches, matching_value, ) {
                (true, Some(v)) => add_search_result(
                    search_result, 
                    SearchResult::ValueNameAndData(Vec::from(&mut *current_path), value.name().to_owned(), v))?,
                (true, None) => add_search_result(
                    search_result,
                    SearchResult::ValueName(Vec::from(&mut *current_path), value.name().to_owned()))?,
                (false, Some(v)) => add_search_result(
                    search_result,
                    SearchResult::ValueData(Vec::from(&mut *current_path), value.name().to_owned(), v),
                )?,
                (false, None) => (),
            }
        }
        Ok(())
    }
}

/// TODO: replace this with the official ThenSome trait as far as it is stable
trait RegviewThenSome<'a, 'b, T> where T: 'b {
    fn regview_then_some(&'a self, v: T) -> Option<T>;
}

impl<'a, 'b> RegviewThenSome<'a, 'b, String> for bool {
    fn regview_then_some(&'a self, v: String) -> Option<String> {
        if *self {
            Some(v)
        } else {
            None
        }
    }
}

fn add_search_result(target: &mut Vec<SearchResult>, search_result: SearchResult) -> Result<()> {
    if target.len() > 999 {
        return Err(anyhow!(SearchError::TooManyResults));
    }
    target.push(search_result);
    Ok(())
}
