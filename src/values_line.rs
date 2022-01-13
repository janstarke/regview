use cursive_table_view::TableViewItem;
use anyhow::Result;
use nt_hive::{Hive, KeyValue, KeyValueData, KeyValueDataType};

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

fn binary_data(data: &KeyValue<&Hive<MmapByteSlice>, MmapByteSlice>) -> String {
    match data.data().unwrap() {
        KeyValueData::Small(v) => format!("{:X?}", v),
        KeyValueData::Big(_) => "BigDataSlices unsupported".to_owned()
    }
}

impl ValuesLine {
    pub fn from(value: &KeyValue<&Hive<MmapByteSlice>, MmapByteSlice>) -> Result<Self> {
        let (datatype, data) = 
        match value.data_type()? {
            KeyValueDataType::RegNone => ("RegNone", "".to_owned()),
            KeyValueDataType::RegSZ=> ("RegSZ", value.string_data()?),
            KeyValueDataType::RegExpandSZ=> ("RegExpandSZ", value.string_data()?),
            KeyValueDataType::RegBinary=> ("RegBinary", binary_data(value)),
            KeyValueDataType::RegDWord=> ("RegDWord", format!("0x{:08x}", value.dword_data()?)),
            KeyValueDataType::RegDWordBigEndian=> ("RegDWordBigEndian", format!("0x{:08X}", u32::from_be(value.dword_data()?))),
            KeyValueDataType::RegLink=> ("RegLink", "not supported".to_owned()),
            KeyValueDataType::RegMultiSZ=> ("RegMultiSZ", value.multi_string_data()?.join("|")),
            KeyValueDataType::RegResourceList=> ("RegResourceList", "not supported".to_owned()),
            KeyValueDataType::RegFullResourceDescriptor=> ("RegFullResourceDescriptor", "not supported".to_owned()),
            KeyValueDataType::RegResourceRequirementsList=> ("RegResourceRequirementsList", "not supported".to_owned()),
            KeyValueDataType::RegQWord=> ("RegQWord", format!("0x{:016x}", value.qword_data()?)),
        };

        Ok(Self {
            name: value.name()?.to_string_lossy(),
            data,
            datatype: datatype.to_owned()
        })
    }

    pub fn new(name: String, data: String, datatype: String) -> Self {
        Self {
            name, data, datatype
        }
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
