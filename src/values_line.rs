use std::fmt::Binary;

use cursive_table_view::TableViewItem;
use anyhow::Result;
use nt_hive::{Hive, KeyNode, KeyValue};

use crate::mmap_byteslice::MmapByteSlice;

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
    pub fn from(value: &KeyValue<&Hive<MmapByteSlice>, MmapByteSlice>) -> Result<Self> {
        let (datatype, data) = 
        match value.data_type() {
            RegNone => ("RegNone", "".to_owned()),
            RegSZ=> ("RegSZ", value.string_data()?),
            RegExpandSZ=> ("RegExpandSZ", value.string_data()?),
            RegBinary=> ("RegBinary", "not supported".to_owned()),
            RegDWord=> ("RegDWord", format!("0x{:08x}", value.dword_data()?)),
            RegDWordBigEndian=> ("RegDWordBigEndian", format!("0x{:08X}", u32::from_be(value.dword_data()?))),
            RegLink=> ("RegLink", "not supported".to_owned()),
            RegMultiSZ=> ("RegMultiSZ", value.multi_string_data()?.join("|")),
            RegResourceList=> ("RegResourceList", "not supported".to_owned()),
            RegFullResourceDescriptor=> ("RegFullResourceDescriptor", "not supported".to_owned()),
            RegResourceRequirementsList=> ("RegResourceRequirementsList", "not supported".to_owned()),
            RegQWord=> ("RegQWord", format!("0x{:016x}", value.qword_data()?)),
        };

        Ok(Self {
            name: value.name()?.to_string_lossy(),
            data,
            datatype: datatype.to_owned()
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
