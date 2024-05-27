use anyhow::Result;
use strum::{
    EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr,
    VariantNames,
};

#[derive(
    EnumString, EnumCount, EnumDiscriminants, EnumIter, EnumIs, IntoStaticStr, VariantNames, Debug,
)]
enum MyEnum {
    A,
    B,
    C,
    D,
}

fn main() -> Result<()> {
    MyEnum::iter().for_each(|e| println!("{:?}", e));
    println!("EnumCount: {:?}", MyEnum::COUNT);
    MyEnum::VARIANTS.iter().for_each(|e| println!("{:?}", e));

    let a = MyEnum::A;

    println!("a is {:?}", a.is_a());
    Ok(())
}
