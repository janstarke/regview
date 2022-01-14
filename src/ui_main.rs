use anyhow::Result;
use cursive::event;
use cursive::menu::MenuTree;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::{DebugView, DummyView};
use cursive::Cursive;
use cursive::{
    views::{Dialog, EditView, LinearLayout, OnEventView, Panel, ResizedView, TextView, ViewRef},
    CursiveRunnable,
};
use cursive_table_view::TableView;
use std::cell::RefCell;
use std::rc::Rc;

use crate::registry_hive::{RegistryHive, SearchResult};
use crate::values_line::*;
use crate::keys_line::*;

static NAME_KEYS_TABLE: &str = "keys_table";
static NAME_VALUES_TABLE: &str = "values_table";
static NAME_PATH_LINE: &str = "path_line";
static NAME_DEBUG_VIEW: &str = "debug_view";
static NAME_SEARCH_REGEX: &str = "search_regex";

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
            .column(KeysColumn::Name, "Name", |c| {c})
            //.column(KeysColumn::LastWritten, "Last written", |c| c.width(20))
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
                        .min_width(48)
                        .max_width(64),
                )
                .title("Keys"),
            )
            .child(DummyView)
            .child(
                Panel::new(details_table.with_name(NAME_VALUES_TABLE).full_screen())
                    .title("Values"),
            );

        let root_view = LinearLayout::vertical()
            .child(TextView::new("").with_name(NAME_PATH_LINE))
            .child(reg_view)
            .child(
                Panel::new(
                    DebugView::new()
                        .with_name(NAME_DEBUG_VIEW)
                        .min_height(3)
                        .max_height(10),
                )
                .title("Logging"),
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

    fn on_find_next(siv: &mut Cursive) {
        let search_regex = {
            let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
            user_data.search_regex.clone()
        };

        if let Some(search_regex) = search_regex {
            if !search_regex.is_empty() {
                let search_result = {
                    let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
                    let hive = &user_data.hive;
                    hive.borrow_mut().find_regex(&search_regex).unwrap()
                };

                if matches!(search_result, SearchResult::None) {
                    siv.add_layer(Dialog::info("nothing found"));
                    return;
                }

                let user_data: &mut RegviewUserdata = siv.user_data().unwrap();
                let hive = &user_data.hive;
                let (new_items, key_name) = match search_result {
                    SearchResult::KeyName(path) => (
                        hive.borrow_mut().select_path(&path).unwrap(),
                        path.last().and_then(|s| Some(s.to_owned())),
                    ),
                    SearchResult::ValueName(path, _) => (
                        hive.borrow_mut().select_path(&path).unwrap(),
                        path.last().and_then(|s| Some(s.to_owned())),
                    ),
                    SearchResult::ValueData(path, _) => (
                        hive.borrow_mut().select_path(&path).unwrap(),
                        path.last().and_then(|s| Some(s.to_owned())),
                    ),
                    _ => {
                        panic!("this should have been handled some lines above");
                    }
                };

                let mut keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
                    siv.find_name(NAME_KEYS_TABLE).unwrap();
                keys_table.clear();

                let selection_index = if let Some(kn) = key_name {
                    new_items.iter().position(|i| i.name() == kn)
                } else {
                    None
                };

                keys_table.set_items(new_items);
                if let Some(index) = selection_index {
                    keys_table.set_selected_item(index);
                }
                keys_table.sort();
            }
        }
    }

    fn on_submit(siv: &mut Cursive, _: usize, index: usize) {
        let mut keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
            siv.find_name(NAME_KEYS_TABLE).unwrap();
        let user_data: &RegviewUserdata = siv.user_data().unwrap();
        let hive = &user_data.hive;
        let selected_node = hive.borrow().selected_node();
        let mut select_node = selected_node.is_some();
        let new_items = match keys_table.borrow_item(index) {
            None => return,
            Some(item) => {
                if item.is_parent() {
                    select_node = select_node & true;
                    hive.borrow_mut().leave().unwrap()
                } else {
                    hive.borrow_mut().enter(item.name()).unwrap()
                }
            }
        };

        keys_table.clear();
        let selection_index = if select_node {
            new_items
                .iter()
                .position(|i| i.name() == selected_node.as_ref().unwrap())
        } else {
            None
        };

        keys_table.set_items(new_items);
        if let Some(index) = selection_index {
            keys_table.set_selected_item(index);
        }
        keys_table.sort();

        let path = hive.borrow().path();
        siv.call_on_name(NAME_PATH_LINE, |l: &mut TextView| l.set_content(path));
    }

    fn on_select(siv: &mut Cursive, _: usize, index: usize) {
        let keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
            siv.find_name(NAME_KEYS_TABLE).unwrap();

        let new_items = match keys_table.borrow_item(index) {
            None => Vec::new(),
            Some(item) => {
                if item.is_parent() {
                    vec![ValuesLine::new(
                        "parent key".to_owned(),
                        "".to_owned(),
                        "".to_owned(),
                    )]
                } else {
                    let user_data: &RegviewUserdata = siv.user_data().unwrap();
                    let hive = &user_data.hive;
                    hive.borrow().key_values(item.name()).unwrap()
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
