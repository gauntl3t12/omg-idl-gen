use serde_derive::{Serialize, Deserialize};

#[allow(dead_code, non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Foo {
    LABEL0{ l: i32, },
    LABEL1{ s: i16, },
    LABEL2{ s: i16, },
    default{ o: u8, },
}
//
// TODO custom de-/serializer
//
