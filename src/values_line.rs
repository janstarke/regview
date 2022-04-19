use cursive_table_view::TableViewItem;
use anyhow::Result;
use nt_hive2::{KeyValue, RegistryValue};

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
    pub fn from(value: &KeyValue) -> Result<Self> {
        let (datatype, data) = 
        match &value.value() {
            RegistryValue::RegNone 
                => ("RegNone", "".to_owned()),
            RegistryValue::RegUnknown 
                => ("RegUnknown", "".to_owned()),
            RegistryValue::RegSZ(val) 
                => ("RegSZ", val.to_owned()),
            RegistryValue::RegExpandSZ(val) 
                => ("RegExpandSZ", val.to_owned()),
            RegistryValue::RegBinary(val) 
                => ("RegBinary", format!("{:X?}", val)),
            RegistryValue::RegDWord(val) 
                => ("RegDWord", format!("0x{:08x}", val)),
            RegistryValue::RegDWordBigEndian(val) 
                => ("RegDWordBigEndian", format!("0x{:08X}", u32::from_be(*val))),
            RegistryValue::RegLink(val) 
                => ("RegLink", val.to_owned()),
            RegistryValue::RegMultiSZ(val) 
                => ("RegMultiSZ", val.join("|")),
            RegistryValue::RegResourceList(val) 
                => ("RegResourceList", val.to_owned()),
            RegistryValue::RegFullResourceDescriptor(val) 
                => ("RegFullResourceDescriptor", val.to_owned()),
            RegistryValue::RegResourceRequirementsList(val) 
                => ("RegResourceRequirementsList", val.to_owned()),
            RegistryValue::RegQWord(val) 
                => ("RegQWord", format!("0x{:016x}", val)),
            RegistryValue::RegFileTime 
                => ("RegFileTime", "not supported".to_owned()),
        };

        Ok(Self {
            name: value.name().to_owned(),
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
