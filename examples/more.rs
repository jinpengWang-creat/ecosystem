use anyhow::Result;

fn main() -> Result<()> {
    extern crate derive_more;
    use derive_more::{Add, Display, From, Into};

    #[derive(PartialEq, From, Add)]
    struct MyInt(i32);

    #[derive(PartialEq, From, Into)]
    struct Point2D {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, From, Add, Display)]
    enum MyEnum {
        #[display(fmt = "int: {}", _0)]
        Int(i32),
        Uint(u32),
        #[display(fmt = "nothing")]
        Nothing,
    }

    assert!(MyInt(11) == MyInt(5) + 6.into());
    assert!((5, 6) == Point2D { x: 5, y: 6 }.into());
    assert!(MyEnum::Int(15) == (MyEnum::Int(8) + 7.into()).unwrap());
    assert!(MyEnum::Int(15).to_string() == "int: 15");
    assert!(MyEnum::Uint(42).to_string() == "42");
    assert!(MyEnum::Nothing.to_string() == "nothing");

    let e: MyEnum = 10i32.into();
    let e1: MyEnum = 20u32.into();
    let e2 = e + e1;
    println!("e: {}, e1: {}, e2: {:?}", e, e1, e2);
    Ok(())
}
