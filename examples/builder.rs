#![allow(unused, dead_code, non_snake_case)]
use anyhow::Result;

use crate::user1::{User, UserBuilder};
fn main() -> Result<()> {
    let user = User::build();
    println!("{:?}", user);
    Ok(())
}

mod user1 {

    use std::marker::PhantomData;

    use derive_builder::Builder;
    #[derive(Debug, Default, Builder)]
    #[builder(setter(into, strip_option), pattern = "owned")]
    #[builder(build_fn(private))]
    pub struct User {
        name: String,
        #[builder(setter(name = "setAge"))]
        age: u8,

        email: Option<String>,
        #[builder(setter(each(name = "skill", into)))]
        skills: Vec<String>,

        #[builder(setter(skip))]
        height: i32,
    }

    impl User {
        pub fn build() -> Self {
            UserBuilder::default()
                .name("tom")
                .setAge(11)
                .email("hhhh")
                .skill("hhh")
                .skill("bbbb")
                .build()
                .unwrap()
        }
    }
}
