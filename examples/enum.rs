use anyhow::Result;
use strum::{EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr, VariantArray, VariantNames};

#[derive(Debug, EnumString, EnumCount, EnumDiscriminants, EnumIter, 
  EnumIs, IntoStaticStr, VariantNames)]
enum MyEnum {
  A,
  B(String),
  C,
}

fn main() -> Result<()> {
  println!("{:?}", MyEnum::VARIANTS);
  MyEnum::iter().for_each(|v| println!("{:?}", v));


  Ok(())
}