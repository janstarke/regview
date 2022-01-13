use anyhow::Result;
use cursive::view::{Nameable, Resizable, SizeConstraint};
use cursive::views::{BoxedView, DebugView, DummyView};
use cursive::Cursive;
use cursive::{
    views::{LinearLayout, Panel, ResizedView, TextView, ViewRef},
    CursiveRunnable,
};
use cursive_table_view::TableView;
use std::cell::RefCell;
use std::rc::Rc;

use crate::keys_line::*;
use crate::registry_hive::RegistryHive;
use crate::values_line::*;

static NAME_KEYS_TABLE: &str = "keys_table";
static NAME_VALUES_TABLE: &str = "values_table";
static NAME_PATH_LINE: &str = "path_line";
static NAME_DEBUG_VIEW: &str = "debug_view";

pub struct UIMain {
    siv: CursiveRunnable,
}

impl UIMain {
    pub fn new(hive: Rc<RefCell<RegistryHive>>) -> Self {
        let mut siv = cursive::default();

        siv.set_user_data(Rc::clone(&hive));
        let mut me = Self { siv: siv };
        me.construct();
        assert!(me.siv.user_data::<Rc<RefCell<RegistryHive>>>().is_some());
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
            .child(Panel::new(
                keys_table
                    .with_name(NAME_KEYS_TABLE)
                    .full_height()
                    .min_width(48)
                    .max_width(64),
            ).title("Keys"))
            .child(DummyView)
            .child(Panel::new(
                details_table.with_name(NAME_VALUES_TABLE).full_screen(),
            ).title("Values"));

        let root_view = LinearLayout::vertical()
            .child(TextView::new("").with_name(NAME_PATH_LINE))
            .child(reg_view)
            .child(Panel::new(
                DebugView::new()
                    .with_name(NAME_DEBUG_VIEW)
                    .min_height(3)
                    .max_height(10),
            ).title("Logging"));

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
    }

    fn on_submit(siv: &mut Cursive, _: usize, index: usize) {
        let mut keys_table: ViewRef<TableView<KeysLine, KeysColumn>> =
            siv.find_name(NAME_KEYS_TABLE).unwrap();
            
        let hive: &Rc<RefCell<RegistryHive>> = siv.user_data().unwrap();
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
            None => {
                Vec::new()
            }
            Some(item) => {
                if item.is_parent() {
                    vec![ValuesLine::new("parent key".to_owned(), "".to_owned(), "".to_owned())]
                } else {
                    let hive: &Rc<RefCell<RegistryHive>> = siv.user_data().unwrap();
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
            let hive: &Rc<RefCell<RegistryHive>> = self.siv.user_data().unwrap();
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
