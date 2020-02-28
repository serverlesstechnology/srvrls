use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt::Debug;
use std::cmp::PartialEq;

pub trait Event: erased_serde::Serialize + Debug {
    fn name(&self) -> String;
    fn from_json(&mut self, value: Value) -> Result<(),serde_json::Error>;
}

serialize_trait_object!(Event);

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestDto {
    pub id: String,
    pub name: String,
}
impl Event for TestDto{
    fn name(&self) -> String {
        "TestDto".to_string()
    }

    fn from_json(&mut self, value: Value)  -> Result<(),serde_json::Error>{
        *self = serde_json::from_value(value).unwrap();
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestDtoB {
    pub id: String,
    pub email: String,
}
impl Event for TestDtoB{
    fn name(&self) -> String {
        "TestDtoB".to_string()
    }
    fn from_json(&mut self, value: Value) -> Result<(),serde_json::Error> {
        *self = serde_json::from_value(value).unwrap();
        Ok(())
    }

}