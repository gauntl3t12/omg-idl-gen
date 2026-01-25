use serde_derive::{Serialize, Deserialize};

#[allow(dead_code, non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Foo {
    VARIANT0,
    VARIANT1,
    VARIANT2,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FooError;

impl std::str::FromStr for Foo {
    type Err = FooError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "VARIANT0" => Ok(Foo::VARIANT0),
            "VARIANT1" => Ok(Foo::VARIANT1),
            "VARIANT2" => Ok(Foo::VARIANT2),
            _ => Err(FooError),
        }
    }
}

impl std::fmt::Display for Foo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let enum_str = match self {
            Foo::VARIANT0 => "VARIANT0",
            Foo::VARIANT1 => "VARIANT1",
            Foo::VARIANT2 => "VARIANT2",
        };
        write!(f, "{enum_str}")
    }
}
