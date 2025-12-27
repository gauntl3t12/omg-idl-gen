#[allow(unused_imports)]
use std::vec::Vec;
#[allow(unused_imports)]
use serde_derive::{Serialize, Deserialize};

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub enum Foo {
    VARIANT0,
    VARIANT1,
    VARIANT2,
}

impl Foo {
    /// Method to convert string to enum variant
    pub fn from_str(value: &str) -> Option<Foo> {
        match value {
            "VARIANT0" => Some(Foo::VARIANT0),
            "VARIANT1" => Some(Foo::VARIANT1),
            "VARIANT2" => Some(Foo::VARIANT2),
            _ => None,
        }
    }

    /// Method to convert enum variant to &str
    pub fn to_str(&self) -> &str {
        match self {
            Foo::VARIANT0 => "VARIANT0",
            Foo::VARIANT1 => "VARIANT1",
            Foo::VARIANT2 => "VARIANT2",
        }
    }
}
