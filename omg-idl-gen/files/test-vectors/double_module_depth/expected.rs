
#[allow(non_snake_case)]
pub mod A {

    #[allow(non_snake_case)]
    pub mod B {
        use serde_derive::{Serialize, Deserialize};

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #[derive(Serialize, Deserialize)]
        #[derive(Clone, Debug)]
        pub struct Foo {
            pub m_l1: i32,
            pub m_l2: i32,
            pub m_d: f64,
        }

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

    }

}
