use anyhow::Result;
use cursive::event;
use cursive::menu::MenuTree;
use cursive::view::{Nameable, Resizable, SizeConstraint, Selector};
use cursive::views::DummyView;
use cursive::Cursive;
use cursive::{
    views::{Dialog, EditView, LinearLayout, OnEventView, Panel, ResizedView, TextView, ViewRef},
    CursiveRunnable,
};
use cursive_table_view::TableView;
use std::cell::RefCell;
use std::rc::Rc;

use crate::keys_line::*;
use crate::registry_hive::{RegistryHive};
use crate::values_line::*;
use crate::search_result::*;

static NAME_KEYS_TABLE: &str = "keys_table";
static NAME_VALUES_TABLE: &str = "values_table";
static NAME_PATH_LINE: &str = "path_line";
static NAME_SEARCH_REGEX: &str = "search_regex";
static NAME_SEARCH_RESULTS: &str = "search_results";
static NAME_SEARCH_PANEL: &str = "search_panel";

pub struct UIMain {
    siv: CursiveRunnable,
}

struct RegviewUserdata {
    hive: Rc<RefCell<RegistryHive>>,
    search_regex: Option<String>,
}

impl UIMain {
    pub fn new(hive: Rc<RefCell<RegistryHive>>) -> Self {
        let mut siv = cursive::default();

        let user_data = RegviewUserdata {
            hive,
            search_regex: None,
        };
        siv.set_user_data(user_data);
        let mut me = Self { siv: siv };
        me.construct();
        me
    }

    fn construct(&mut self) {
        self.siv.add_global_callback('q', |s| s.quit());

        let mut keys_table = TableView::<KeysLine, KeysColumn>::new()
            .column(KeysColumn::NodeType, "", |c|{c.width(1)})
            .column(KeysColumn::Name, "Name", |c| {c})
            .column(KeysColumn::LastWritten, "Timestamp", |c| {c}.width(20))
        ;

        keys_table.set_on_submit(UIMain::on_submit);
        keys_table.set_on_select(UIMain::on_select);

        let details_table = TableView::<ValuesLine, ValuesColumn>::new()
            .column(ValuesColumn::Name, "Name", |c| c.width(24))
            .column(ValuesColumn::Data, "Value", |c| c)
            .column(ValuesColumn::Type, "Datatype", |c| c.width(16));

        //details_table.set_enabled(false);

        let reg_view = LinearLayout::horizontal()
            .child(
                Panel::new(
                    keys_table
                        .with_name(NAME_KEYS_TABLE)
                        .full_height()
                        .min_width(53)
                        .max_width(64),
                )
                .title("Keys"),
            )
            .child(DummyView)
            .child(
                Panel::new(details_table.with_name(NAME_VALUES_TABLE).full_screen())
                    .title("Values"),
            );

        let mut search_results = TableView::<SearchResultLine, SearchResultColumns>::new()
            .column(SearchResultColumns::KeyName, "Key", |c| c.width(48))
            .column(SearchResultColumns::ValueName, "Value", |c| c.width(16))
            .column(SearchResultColumns::ValueData, "Value", |c| c);

        search_results.set_on_submit(UIMain::on_select_search_result);

        let root_view = LinearLayout::vertical()
            .child(TextView::new("").with_name(NAME_PATH_LINE))
            .child(reg_view)
            .child(
                Panel::new(search_results.with_name(NAME_SEARCH_RESULTS).full_width().max_height(10).min_height(10))
                .title("Search results")
                .with_name(NAME_SEARCH_PANEL)
            );

        self.siv.add_layer(
            Panel::new(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Full,
                root_view,
            ))
            .title(format!(
                "{} v{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
        );

        self.siv.menubar().add_subtree(
            "File",
            MenuTree::new()
                .leaf("Find", UIMain::on_find)
                .delimiter()
                .leaf("Quit", |s| s.quit()),
        );
        self.siv.set_autohide_menu(false);
        self.siv
            .add_global_callback(event::Key::Esc, |s| s.select_menubar());
        self.siv.add_global_callback('f', |s| s.select_menubar());

        self.siv
            .add_global_callback(event::Key::F3, UIMain::on_find);
    }

    fn on_find(siv: &mut Cursive) {
        let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
        let edit_view = EditView::new()
            .content(
                user_data
                    .search_regex
                    .as_ref()
                    .or(Some(&"".to_owned()))
                    .unwrap(),
            )
            .with_name(NAME_SEARCH_REGEX)
            .min_width(32);

        let okay_handler = |s: &mut Cursive| {
            Self::store_search_regex(s);
            s.pop_layer();
            Self::on_find_next(s);
        };
        let mut find_dialog = Dialog::around(
            LinearLayout::vertical().child(
                LinearLayout::horizontal()
                    .child(TextView::new("Search regex:"))
                    .child(OnEventView::new(edit_view).on_event(event::Key::Enter, okay_handler)),
            ),
        );
        find_dialog.add_button("Find", okay_handler);
        find_dialog.add_button("Cancel", |s| {
            s.pop_layer();
        });
        siv.add_layer(find_dialog);
    }

    fn store_search_regex(siv: &mut Cursive) {
        let edit_view: ViewRef<EditView> = siv.find_name(NAME_SEARCH_REGEX).unwrap();
        let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
        user_data.search_regex = Some(edit_view.get_content().to_string());
    }

    fn display_error(siv: &mut Cursive, error: anyhow::Error) {
        siv.add_layer(Dialog::info(format!("ERROR: {}", error)));
    }

    fn on_find_next(siv: &mut Cursive) {
        let search_regex = {
            let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
            user_data.search_regex.clone()
        };

        if let Some(search_regex) = search_regex {
            if !search_regex.is_empty() {
                let search_result = {
                    let search_result = {
                        let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
                        let hive = &user_data.hive;
                        hive.borrow_mut().find_regex(&search_regex)
                    };
                    match search_result {
                        Ok(result) => result,
                        Err(why) => {
                            UIMain::display_error(siv, why);
                            return;
                        }
                    }
                };

                siv.call_on_name(NAME_SEARCH_RESULTS, |sr_table: &mut TableView<SearchResultLine, SearchResultColumns>| {
                    sr_table.clear();
                    sr_table.set_items(search_result.into_iter().map(SearchResultLine::from).collect());

                });
                let _ = siv.focus(&Selector::Name(NAME_SEARCH_RESULTS));
            }
        }
    }

    fn on_select_search_result(siv: &mut Cursive, _: usize, index: usize) {
        let search_results_table: ViewRef<TableView<SearchResultLine, SearchResultColumns>> =siv.find_name(NAME_SEARCH_RESULTS).unwrap();
        let mut selected_line = match search_results_table.borrow_item(index) {
            None => return,
            Some(item) => item.clone()
        };

        let displayed_path = selected_line.path.join("\\");
        let my_key = selected_line.path.pop();

        let new_items = {
            let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
            let hive = &user_data.hive;
            hive.borrow_mut().select_path(&selected_line.path)
        };

        let new_items = match new_items {
            Ok(new_items) => new_items,
            Err(why) => {
                Self::display_error(siv, why);
                return;
            }
        };

        let mut keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
        siv.find_name(NAME_KEYS_TABLE).unwrap();
        keys_table.clear();

        let selection_index = match my_key {
            Some(kn) =>  new_items.iter().position(|i| i.name() == kn),
            None => None
        };

        keys_table.set_items(new_items);
        let selected_item = if let Some(index) = selection_index {
            keys_table.set_selected_item(index);
            keys_table.borrow_item(index) 
        } else { None };

        siv.call_on_name(NAME_PATH_LINE, |l: &mut TextView| {
            l.set_content(displayed_path)
        });


        let new_items = match selected_item {
            None => Vec::new(),
            Some(item) => {
                if item.is_parent() {
                    vec![]
                } else {
                    let user_data: &RegviewUserdata = siv.user_data().unwrap();
                    let hive = &user_data.hive;
                    hive.borrow().key_values(item.name()).unwrap()
                }
            }
        };

        let value_index = match selected_line.value_name {
            None => None,
            Some(vn) => new_items.iter()
                .position(|vl| vl.name() == vn)
        };

        let mut values_table: ViewRef<TableView<ValuesLine, ValuesColumn>> =
            siv.find_name(NAME_VALUES_TABLE).unwrap();
        values_table.clear();
        values_table.set_items(new_items);
        if let Some(vi) = value_index {
            values_table.set_selected_item(vi);
        }

        keys_table.sort();

    }

    fn on_submit(siv: &mut Cursive, _: usize, index: usize) {
        let mut keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
            siv.find_name(NAME_KEYS_TABLE).unwrap();
        let user_data: &RegviewUserdata = siv.user_data().unwrap();
        let hive = &user_data.hive;
        let selected_node_name = hive.borrow().selected_node();
        let mut select_node = selected_node_name.is_some();
        let selected_node = match keys_table.borrow_item(index) {
            None => return,
            Some(item) => item,
        };

        let new_items = if selected_node.is_parent() {
            select_node = select_node & true;
            let result = hive.borrow_mut().leave();
            match result {
                Err(why) => {
                    Self::display_error(siv, why);
                    return;
                }
                Ok(value) => value
            }
        } else {
            if selected_node.is_leaf_node() {
                return;
            } else {
                let result = hive.borrow_mut().enter(selected_node.name());
                match result  {
                    Err(why) => {
                        Self::display_error(siv, why);
                        return;
                    }
                    Ok(value) => value
                }
            }
        };

        keys_table.clear();
        let selection_index = if select_node {
            new_items
                .iter()
                .position(|i| i.name() == selected_node_name.as_ref().unwrap())
        } else {
            None
        };

        keys_table.set_items(new_items);
        if let Some(index) = selection_index {
            keys_table.set_selected_item(index);
        }
        keys_table.sort();

        let path = hive.borrow().path().join("\\");
        siv.call_on_name(NAME_PATH_LINE, |l: &mut TextView| l.set_content(path));

    }

    fn on_select(siv: &mut Cursive, _: usize, index: usize) {
        let keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
            siv.find_name(NAME_KEYS_TABLE).unwrap();

        let new_items = match keys_table.borrow_item(index) {
            None => Vec::new(),
            Some(item) => {
                if item.is_parent() {
                    vec![]
                } else {
                    let user_data: &RegviewUserdata = siv.user_data().unwrap();
                    let hive = &user_data.hive;

                    let key_values = hive.borrow().key_values(item.name());
                    match key_values {
                        Err(why) => {
                            Self::display_error(siv, why);
                            return;
                        }
                        Ok(value) => value
                    }
                }
            }
        };

        let mut values_table: ViewRef<TableView<ValuesLine, ValuesColumn>> =
            siv.find_name(NAME_VALUES_TABLE).unwrap();
        values_table.clear();
        values_table.set_items(new_items);
    }

    pub fn run(&mut self) -> Result<()> {
        let items = {
            let user_data: &RegviewUserdata = self.siv.user_data().unwrap();
            let hive = &user_data.hive;
            hive.borrow().current_keys()?
        };
        self.siv.call_on_name(
            NAME_KEYS_TABLE,
            |v: &mut TableView<KeysLine, KeysColumn>| v.set_items(items),
        );
        self.siv.run();
        Ok(())
    }
}
