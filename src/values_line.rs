use cursive_table_view::TableViewItem;
use rwinreg::vk::ValueKey;
use rwinreg::vk::Data;
use anyhow::Result;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ValuesColumn {
    Name,
    Data,
    Type
}

#[derive(Clone, Debug)]
pub struct ValuesLine {
    name: String,
    data: String,
    datatype: String
}

impl ValuesLine {
    pub fn from(value: &ValueKey) -> Result<Self> {
        let (data, datatype) =
            match value.decode_data()? {
                None => { ("NONE".to_owned(), "".to_owned()) }
                Some(data) => {
                    match data {
                        Data::None => ("NONE".to_owned(), "NONE".to_owned()),
                        Data::String(s) => (s, "String".to_owned()),
                        Data::Int32(i) => (i.to_string(), "Int32".to_owned())

                    }
                }
            };
        Ok(Self {
            name: value.get_name().to_owned(),
            data: data,
            datatype: datatype
        })
    }
}

impl TableViewItem<ValuesColumn> for ValuesLine {
    fn to_column(&self, column: ValuesColumn) -> String {
        match column {
            ValuesColumn::Name => self.name.clone(),
            ValuesColumn::Data => self.data.clone(),
            ValuesColumn::Type => self.datatype.clone()
        }
    }

    fn cmp(&self, other: &Self, column: ValuesColumn) -> std::cmp::Ordering
    where 
        Self: Sized,
    {
        match column {
            ValuesColumn::Name => self.name.cmp(&other.name),
            ValuesColumn::Data => self.data.cmp(&other.data),
            ValuesColumn::Type => self.datatype.cmp(&other.datatype),
        }
    }
}
