use serde::{Serialize, Deserialize};
use serde_json::{Value, Error};
use std::fmt::Debug;
use std::cmp::PartialEq;
use std::any::Any;
use serde_json::error::Category;

pub struct AggregateError {
    message: String
}
impl AggregateError {
    fn new(message: &str) -> Self{
        AggregateError{ message: message.to_string() }
    }
}
impl From<serde_json::error::Error> for AggregateError {
    fn from(err: Error) -> Self {
        match err.classify() {
            Category::Syntax => AggregateError{ message: "invalid json".to_string() },
            Category::Io |
            Category::Data |
            Category::Eof => AggregateError{ message: "fail".to_string() },
        }
    }
}
struct AggregateId(String);


pub struct ProjectAggregate {
    id: AggregateId,
    full_name: String,
    email: String,
}
impl ProjectAggregate {
    fn apply(&mut self, se: Vec<SerializedEvent>) -> Result<(),AggregateError> {
        for event in se {
            match event.name.as_str() {
                "TestDto" => {
                    let event: TestDto = serde_json::from_value(event.payload)?;
                    self.apply_test_dto(event)?
                }
                "TestDtoB" => {
                    let event: TestDtoB = serde_json::from_value(event.payload)?;
                    self.apply_test_dto_b(event)?
                }
                _ => return Err(AggregateError::new("unconfigured event"))
            }
        }
        Ok(())
    }

    fn apply_test_dto(&mut self, event: TestDto) -> Result<(),AggregateError> {
        self.id = AggregateId(event.id);
        self.full_name = event.full_name;
        Ok(())
    }
    fn apply_test_dto_b(&mut self, event: TestDtoB) -> Result<(),AggregateError> {
        self.id = AggregateId(event.id);
        self.email = event.email;
        Ok(())
    }
}

pub struct SerializedEvent {
    name: String,
    aggregate_id: AggregateId,
    payload: Value,
}

pub trait Event<T>: erased_serde::Serialize + Debug + Any {
    fn name(&self) -> String;
    fn from_json(&mut self, value: Value) -> Result<(),serde_json::Error>;
}

serialize_trait_object!(Event<ProjectAggregate>);

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct TestDto {
    pub id: String,
    pub full_name: String,
}
impl Event<ProjectAggregate> for TestDto{
    fn name(&self) -> String {
        "TestDto".to_string()
    }

    fn from_json(&mut self, value: Value)  -> Result<(),serde_json::Error>{
        *self = serde_json::from_value(value)?;
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct TestDtoB {
    pub id: String,
    pub email: String,
}
impl Event<ProjectAggregate> for TestDtoB{
    fn name(&self) -> String {
        "TestDtoB".to_string()
    }
    fn from_json(&mut self, value: Value) -> Result<(),serde_json::Error> {
        *self = serde_json::from_value(value)?;
        Ok(())
    }
}