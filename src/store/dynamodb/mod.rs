//! # DynamoDB store implementation
//!
//! Store implementation using the AWS SDK for DynamoDB.

use super::{Store, StoreDelete, StoreGet, StorePut};
use crate::{
    error::Error,
    model::{Test, TestRun},
};
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::aws_lambda_events::serde_json::Value;
use serde::{de::value::MapDeserializer, Deserialize};
use std::collections::HashMap;
use tracing::{info, instrument};

mod ext;
use ext::AttributeValuesExt;

trait ToStringStringMap {
    fn to_string_string_map(&self) -> HashMap<String, String>;
}

impl ToStringStringMap for HashMap<String, Value> {
    fn to_string_string_map(&self) -> HashMap<String, String> {
        self.iter()
            .map(|(k, v)| {
                let v = match v.clone() {
                    e @ Value::Number(_) | e @ Value::Bool(_) => e.to_string(),
                    Value::String(s) => s,
                    _ => {
                        println!(r#"Warning : Can not convert field : "{}'s value to String, It will be empty string."#, k);
                        "".to_string()
                    }
                };

                (k.clone(), v)
            })
            .collect()
    }
}

/// DynamoDB store implementation.
pub struct DynamoDBStore {
    client: Client,
    table_name: String,
}

impl DynamoDBStore {
    pub fn new(client: Client, table_name: String) -> DynamoDBStore {
        DynamoDBStore { client, table_name }
    }
}

impl Store for DynamoDBStore {}

#[async_trait]
impl StoreGet for DynamoDBStore {
    /// Get item
    #[instrument(skip(self))]
    async fn get(&self, id: &str) -> Result<Option<TestRun>, Error> {
        info!("Getting item with id '{}' from DynamoDB table", id);
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(match res.item {
            Some(item) => Some(item.try_into()?),
            None => None,
        })
    }
}

#[async_trait]
impl StorePut for DynamoDBStore {
    /// Create or update an item
    #[instrument(skip(self))]
    async fn put(&self, testrun: &TestRun) -> Result<(), Error> {
        info!("Putting item with id '{}' into DynamoDB table", testrun.id);
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(testrun.into()))
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait]
impl StoreDelete for DynamoDBStore {
    /// Delete item
    #[instrument(skip(self))]
    async fn delete(&self, id: &str) -> Result<(), Error> {
        info!("Deleting item with id '{}' from DynamoDB table", id);
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(())
    }
}

impl From<&TestRun> for HashMap<String, AttributeValue> {
    /// Convert a &Product into a DynamoDB item
    fn from(value: &TestRun) -> HashMap<String, AttributeValue> {
        let payload: String = serde_json::to_string(&value.files).unwrap();

        let mut retval = HashMap::new();
        retval.insert("id".to_owned(), AttributeValue::S(value.id.clone()));
        retval.insert(
            "language".to_owned(),
            AttributeValue::S(value.language.to_owned()),
        );
        retval.insert("payload".to_owned(), AttributeValue::S(payload));
        retval.insert(
            "status".to_owned(),
            AttributeValue::S(value.status.to_owned()),
        );

        retval
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for TestRun {
    type Error = Error;

    /// Try to convert a DynamoDB item into a TestRun
    ///
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        let files: HashMap<String, String> =
            serde_json::from_str(value.get("files").unwrap().as_s().unwrap()).unwrap();

        let tests: Vec<Test> =
            serde_json::from_str(value.get("tests").unwrap().as_s().unwrap()).unwrap();

        Ok(TestRun {
            id: value
                .get_s("id")
                .ok_or(Error::InternalError("Missing id"))?,
            language: value
                .get_s("language")
                .ok_or(Error::InternalError("Missing language"))?,
            files,
            status: value
                .get_s("status")
                .ok_or(Error::InternalError("Missing status"))?,
            tests,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use aws_sdk_dynamodb::{Client, Config, Credentials, Region};
    use aws_smithy_client::{erase::DynConnector, test_connection::TestConnection};
    use aws_smithy_http::body::SdkBody;

    /// Config for mocking DynamoDB
    async fn get_mock_config() -> Config {
        let cfg = aws_config::from_env()
            .region(Region::new("eu-west-1"))
            .credentials_provider(Credentials::new(
                "accesskey",
                "privatekey",
                None,
                None,
                "dummy",
            ))
            .load()
            .await;

        Config::new(&cfg)
    }

    fn get_request_builder() -> http::request::Builder {
        http::Request::builder()
            .header("content-type", "application/x-amz-json-1.0")
            .uri(http::uri::Uri::from_static(
                "https://dynamodb.eu-west-1.amazonaws.com/",
            ))
    }
}
