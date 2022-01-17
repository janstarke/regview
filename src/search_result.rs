use cursive_table_view::TableViewItem;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SearchResultColumns {
    KeyName,
    ValueName,
    ValueData,
}

#[derive(Debug)]
pub enum SearchResult {
    KeyName(Vec<String>),
    ValueName(Vec<String>, String),
    ValueData(Vec<String>, String, String),
    ValueNameAndData(Vec<String>, String, String)
}

#[derive(Debug, Clone)]
pub struct SearchResultLine {
    pub path: Vec<String>,
    pub value_name: Option<String>,
    pub value_data: Option<String>,
}

impl SearchResultLine {
    pub fn from(search_result: SearchResult) -> Self {
        match search_result {
            SearchResult::KeyName(key) => Self {
                path: key,
                value_name: None,
                value_data: None
            },

            SearchResult::ValueName(key, value_name) => Self {
                path: key,
                value_name: Some(value_name),
                value_data: None
            },

            SearchResult::ValueData(key, value_name, data) => Self {
                path: key,
                value_name: Some(value_name),
                value_data: Some(data)
            },

            SearchResult::ValueNameAndData(key, value_name, data) => Self {
                path: key,
                value_name: Some(value_name),
                value_data: Some(data)
            },
        }
    }
}

impl TableViewItem<SearchResultColumns> for SearchResultLine {
    fn to_column(&self, column: SearchResultColumns) -> String {
        match column {
            SearchResultColumns::KeyName => self.path.join("\\"),
            SearchResultColumns::ValueName => (&self.value_name).as_ref().or(Some(&String::new())).unwrap().clone(),
            SearchResultColumns::ValueData => (&self.value_data).as_ref().or(Some(&String::new())).unwrap().clone(),
        }
    }

    fn cmp(&self, other: &Self, column: SearchResultColumns) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            SearchResultColumns::KeyName  =>  self.path.cmp(&other.path),
            SearchResultColumns::ValueName => self.value_name.cmp(&other.value_name),
            SearchResultColumns::ValueData => self.value_data.cmp(&other.value_data)
        }
    }
}