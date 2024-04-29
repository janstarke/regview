use anyhow::Result;
use cursive_table_view::TableViewItem;
use encoding_rs::{UTF_16LE, WINDOWS_1252};
use nt_hive2::{KeyValue, RegistryValue};
use uuid::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ValuesColumn {
    Name,
    Data,
    Type,
}

#[derive(Clone, Debug)]
pub struct ValuesLine {
    name: String,
    data: String,
    datatype: String,
}

impl ValuesLine {
    pub fn from(value: &KeyValue) -> Result<Self> {
        let (datatype, data) = match &value.value() {
            RegistryValue::RegNone => ("RegNone", "".to_owned()),
            RegistryValue::RegUnknown => ("RegUnknown", "".to_owned()),
            RegistryValue::RegSZ(val) => ("RegSZ", val.to_owned()),
            RegistryValue::RegExpandSZ(val) => ("RegExpandSZ", val.to_owned()),
            RegistryValue::RegBinary(val) => {
                // Sometimes, strings are stored as UTF-16LE, so try this:
                let (value, _, is_error) = UTF_16LE.decode(&val[..]);
                if !is_error && value.is_ascii() {
                    ("RegBinary (UTF-16LE)", format!("{value}"))
                } else {
                    // Hmm, maybe this is ASCII? Let's try:
                    let (value, _, is_error) = WINDOWS_1252.decode(&val[..]);
                    if !is_error {
                        if value.starts_with("DMIO:ID:") {
                            if let Ok(id) = Uuid::from_slice_le(&val[8..24]) {
                                ("RegBinary (CP1252)", format!("DMIO:ID:{{{id}}}"))
                            } else {
                                ("RegBinary (CP1252)", format!("{value}"))
                            }
                        } else if value.is_ascii() {
                            ("RegBinary (CP1252)", format!("{value}"))
                        } else {
                            ("RegBinary", format!("{:X?}", val))
                        }
                    } else {
                        ("RegBinary", format!("{:X?}", val))
                    }
                }
            }
            RegistryValue::RegDWord(val) => ("RegDWord", format!("0x{val:08x} ({val})")),
            RegistryValue::RegDWordBigEndian(val) => {
                ("RegDWordBigEndian", format!("0x{:08X}", u32::from_be(*val)))
            }
            RegistryValue::RegLink(val) => ("RegLink", val.to_owned()),
            RegistryValue::RegMultiSZ(val) => ("RegMultiSZ", val.join("|")),
            RegistryValue::RegResourceList(val) => ("RegResourceList", val.to_owned()),
            RegistryValue::RegFullResourceDescriptor(val) => {
                ("RegFullResourceDescriptor", val.to_owned())
            }
            RegistryValue::RegResourceRequirementsList(val) => {
                ("RegResourceRequirementsList", val.to_owned())
            }
            RegistryValue::RegQWord(val) => ("RegQWord", format!("0x{val:016x} ({val})")),
            RegistryValue::RegFileTime => ("RegFileTime", "not supported".to_owned()),
        };

        Ok(Self {
            name: value.name().to_owned(),
            data,
            datatype: datatype.to_owned(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(dead_code)]
    pub fn data(&self) -> &str {
        &self.data
    }

    #[allow(dead_code)]
    pub fn datatype(&self) -> &str {
        &self.datatype
    }
}

impl TableViewItem<ValuesColumn> for ValuesLine {
    fn to_column(&self, column: ValuesColumn) -> String {
        match column {
            ValuesColumn::Name => self.name.clone(),
            ValuesColumn::Data => self.data.clone(),
            ValuesColumn::Type => self.datatype.clone(),
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
