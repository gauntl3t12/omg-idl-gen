
#[allow(non_snake_case)]
pub mod ModuleA {
    use std::vec::Vec;

    #[allow(dead_code, non_camel_case_types)]
    pub type dim1 = [i32;2_usize];

    #[allow(dead_code, non_camel_case_types)]
    pub type seq_long = Vec<i32>;

}
