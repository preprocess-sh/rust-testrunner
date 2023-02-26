//! # DynamoDB Event models
//!
//! Models for the DynamoDB event entrypoint.
//!
//! We cannot use the models provided by the AWS SDK for Rust, as they do not
//! implement the `serde::Serialize` and `serde::Deserialize` traits.

use crate::{
    error::Error,
    model::{Event, Test, TestRun},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBEvent {
    #[serde(rename = "Records")]
    pub records: Vec<DynamoDBRecord>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBRecord {
    #[serde(rename = "awsRegion")]
    pub aws_region: String,

    #[serde(rename = "dynamodb")]
    pub dynamodb: DynamoDBStreamRecord,

    #[serde(rename = "eventID")]
    pub event_id: String,

    #[serde(rename = "eventName")]
    pub event_name: String,

    #[serde(rename = "eventSource")]
    pub event_source: String,

    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: String,

    #[serde(rename = "eventVersion")]
    pub event_version: String,
}

impl TryFrom<&DynamoDBRecord> for Event {
    type Error = Error;

    /// Try converting a DynamoDB record to an event.
    fn try_from(value: &DynamoDBRecord) -> Result<Self, Self::Error> {
        match value.event_name.as_str() {
            "INSERT" => {
                let testrun = (&value.dynamodb.new_image).try_into()?;
                Ok(Event::Created { testrun })
            }
            "MODIFY" => {
                let old = (&value.dynamodb.old_image).try_into()?;
                let new = (&value.dynamodb.new_image).try_into()?;
                Ok(Event::Updated { old, new })
            }
            "REMOVE" => {
                let testrun = (&value.dynamodb.old_image).try_into()?;
                Ok(Event::Deleted { testrun })
            }
            _ => Err(Error::InternalError("Unknown event type")),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DynamoDBStreamRecord {
    #[serde(rename = "ApproximateCreationDateTime", default)]
    pub approximate_creation_date_time: Option<f64>,

    #[serde(rename = "Keys", default)]
    pub keys: HashMap<String, AttributeValue>,

    #[serde(rename = "NewImage", default)]
    pub new_image: HashMap<String, AttributeValue>,

    #[serde(rename = "OldImage", default)]
    pub old_image: HashMap<String, AttributeValue>,

    #[serde(rename = "SequenceNumber")]
    pub sequence_number: String,

    #[serde(rename = "SizeBytes")]
    pub size_bytes: f64,

    #[serde(rename = "StreamViewType")]
    pub stream_view_type: String,
}

/// Attribute Value
///
/// This is a copy of the `AttributeValue` struct from the AWS SDK for Rust,
/// but without blob and `is_`-prefixed methods.
/// See https://docs.rs/aws-sdk-dynamodb/0.0.22-alpha/aws_sdk_dynamodb/model/enum.AttributeValue.html
#[derive(Deserialize, Serialize, Debug)]
pub enum AttributeValue {
    // B(Blob),
    Bool(bool),
    // Bs(Vec<Blob>),
    L(Vec<AttributeValue>),
    M(HashMap<String, AttributeValue>),
    N(String),
    Ns(Vec<String>),
    Null(bool),
    S(String),
    Ss(Vec<String>),
}

impl AttributeValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_l(&self) -> Option<&Vec<AttributeValue>> {
        match self {
            AttributeValue::L(l) => Some(l),
            _ => None,
        }
    }
    pub fn as_m(&self) -> Option<&HashMap<String, AttributeValue>> {
        match self {
            AttributeValue::M(m) => Some(m),
            _ => None,
        }
    }
    pub fn as_n(&self) -> Option<f64> {
        match self {
            AttributeValue::N(n) => n.parse::<f64>().ok(),
            _ => None,
        }
    }
    pub fn as_ns(&self) -> Vec<f64> {
        match self {
            AttributeValue::Ns(ns) => ns.iter().filter_map(|n| n.parse::<f64>().ok()).collect(),
            _ => Default::default(),
        }
    }
    pub fn as_null(&self) -> Option<bool> {
        match self {
            AttributeValue::Null(null) => Some(*null),
            _ => None,
        }
    }
    pub fn as_s(&self) -> Option<&str> {
        match self {
            AttributeValue::S(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_ss(&self) -> Vec<String> {
        match self {
            AttributeValue::Ss(ss) => ss.to_owned(),
            _ => Default::default(),
        }
    }
}

impl TryFrom<&HashMap<String, AttributeValue>> for TestRun {
    type Error = Error;

    /// Try to convert a DynamoDB item into a Product
    ///
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: &HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        // Convert the files attribute to a HashMap
        let files_json: String = value
            .get("files")
            .ok_or(Error::InternalError("Missing files"))?
            .as_s()
            .ok_or(Error::InternalError("files is not a string"))?
            .to_string();

        let files: HashMap<String, String> = serde_json::from_str(&files_json)
            .map_err(|e| Error::InternalError("Couldn't parse HashMap from payload"))?;

        // Convert the tests attribute to a Vec<Test>
        let tests_json: String = value
            .get("tests")
            .ok_or(Error::InternalError("Missing tests"))?
            .as_s()
            .ok_or(Error::InternalError("tests is not a string"))?
            .to_string();

        let tests: Vec<Test> = serde_json::from_str(&tests_json)
            .map_err(|e| Error::InternalError("Couldn't parse HashMap from payload"))?;

        Ok(TestRun {
            id: value
                .get("id")
                .ok_or(Error::InternalError("Missing id"))?
                .as_s()
                .ok_or(Error::InternalError("id is not a string"))?
                .to_string(),
            language: value
                .get("language")
                .ok_or(Error::InternalError("Missing language"))?
                .as_s()
                .ok_or(Error::InternalError("language is not a string"))?
                .to_string(),
            files,
            status: value
                .get("status")
                .ok_or(Error::InternalError("Missing status"))?
                .as_s()
                .ok_or(Error::InternalError("status is not a string"))?
                .to_string(),
            tests,
        })
    }
}
