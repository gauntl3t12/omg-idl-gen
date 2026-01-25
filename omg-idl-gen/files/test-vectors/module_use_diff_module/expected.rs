
#[allow(non_snake_case)]
pub mod A {
    use serde_derive::{Serialize, Deserialize};

    #[allow(dead_code, non_camel_case_types)]
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Foo {
        #[allow(non_snake_case)]
        pub m_l1: i32,
        #[allow(non_snake_case)]
        pub m_l2: i32,
        #[allow(non_snake_case)]
        pub m_d: f64,
    }

    #[allow(dead_code)]
    impl Foo {

        pub fn new(m_l1: i32, m_l2: i32, m_d: f64, ) -> Self {
            Self {
                m_l1,
                m_l2,
                m_d,
            }
        }

        pub fn m_l1(&self) -> &i32 {
            &self.m_l1
        }

        pub fn set_m_l1(&mut self, value: i32) {
            self.m_l1 = value;
        }

        pub fn m_l2(&self) -> &i32 {
            &self.m_l2
        }

        pub fn set_m_l2(&mut self, value: i32) {
            self.m_l2 = value;
        }

        pub fn m_d(&self) -> &f64 {
            &self.m_d
        }

        pub fn set_m_d(&mut self, value: f64) {
            self.m_d = value;
        }

    }

    #[allow(dead_code, non_upper_case_globals)]
    pub const length: i32 = 20;

}

#[allow(non_snake_case)]
pub mod B {
    use std::vec::Vec;

    #[allow(dead_code, non_camel_case_types)]
    pub type FooSeq = Vec<crate::A::Foo>;

    #[allow(dead_code, non_camel_case_types)]
    pub type Foo = [crate::A::Foo;crate::A::length as usize];

}
