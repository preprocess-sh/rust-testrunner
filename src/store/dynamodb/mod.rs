//! # DynamoDB store implementation
//!
//! Store implementation using the AWS SDK for DynamoDB.

use super::{Store, StoreDelete, StoreGet, StorePut};
use crate::{model::TestRun, error::Error};
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::aws_lambda_events::serde_json::Value;
use serde::{Deserialize, de::value::MapDeserializer};
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
    async fn put(&self, test_run: &TestRun) -> Result<(), Error> {
        info!("Putting item with id '{}' into DynamoDB table", test_run.id);
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(test_run.into()))
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
        let payload: String = serde_json::to_string(&value.payload).unwrap();

        let mut retval = HashMap::new();
        retval.insert("id".to_owned(), AttributeValue::S(value.id.clone()));
        retval.insert("language".to_owned(), AttributeValue::S(value.language.to_owned()));
        retval.insert("payload".to_owned(), AttributeValue::S(payload));
        retval.insert("status".to_owned(), AttributeValue::S(value.status.to_owned()));

        retval
    }
}

impl TryFrom<HashMap<String, AttributeValue>> for TestRun {
    type Error = Error;

    /// Try to convert a DynamoDB item into a TestRun
    ///
    /// This could fail as the DynamoDB item might be missing some fields.
    fn try_from(value: HashMap<String, AttributeValue>) -> Result<Self, Self::Error> {
        // Transform the JSON string into a HashMap<String, String>
        let payload: HashMap<String, String> = serde_json::from_str(value.get("payload").unwrap().as_s().unwrap()).unwrap();

        Ok(TestRun {
            id: value
                .get_s("id")
                .ok_or(Error::InternalError("Missing id"))?,
            language: value
                .get_s("language")
                .ok_or(Error::InternalError("Missing language"))?,
            payload,
            status: value
                .get_s("status")
                .ok_or(Error::InternalError("Missing status"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
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

    // #[tokio::test]
    // async fn test_all_next() -> Result<(), Error> {
    //     // GIVEN a DynamoDBStore with a last evaluated key
    //     let conn = TestConnection::new(vec![(
    //         get_request_builder()
    //             .header("x-amz-target", "DynamoDB_20120810.Scan")
    //             .body(SdkBody::from(r#"{"TableName":"test","Limit":20}"#))
    //             .unwrap(),
    //         http::Response::builder()
    //             .status(200)
    //             .body(SdkBody::from(
    //                 r#"{"Items": [], "LastEvaluatedKey": {"id": {"S": "1"}}}"#,
    //             ))
    //             .unwrap(),
    //     )]);
    //     let client =
    //         Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
    //     let store = DynamoDBStore::new(client, "test".to_string());

    //     // WHEN getting all items
    //     let res = store.all(None).await?;

    //     // THEN the response has a next key
    //     assert_eq!(res.next, Some("1".to_string()));
    //     // AND the request matches the expected request
    //     conn.assert_requests_match(&vec![]);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_delete() -> Result<(), Error> {
    //     // GIVEN a DynamoDBStore
    //     let conn = TestConnection::new(vec![(
    //         get_request_builder()
    //             .header("x-amz-target", "DynamoDB_20120810.DeleteItem")
    //             .body(SdkBody::from(
    //                 r#"{"TableName": "test", "Key": {"id": {"S": "1"}}}"#,
    //             ))
    //             .unwrap(),
    //         http::Response::builder()
    //             .status(200)
    //             .body(SdkBody::from("{}"))
    //             .unwrap(),
    //     )]);
    //     let client =
    //         Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
    //     let store = DynamoDBStore::new(client, "test".to_string());

    //     // WHEN deleting an item
    //     store.delete("1").await?;

    //     // THEN the request matches the expected request
    //     conn.assert_requests_match(&vec![]);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_get() -> Result<(), Error> {
    //     // GIVEN a DynamoDBStore with one item
    //     let conn = TestConnection::new(vec![(
    //         get_request_builder()
    //             .header("x-amz-target", "DynamoDB_20120810.GetItem")
    //             .body(SdkBody::from(r#"{"TableName": "test", "Key": {"id": {"S": "1"}}}"#))
    //             .unwrap(),
    //         http::Response::builder()
    //             .status(200)
    //             .body(SdkBody::from(r#"{"Item": {"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.0"}}}"#))
    //             .unwrap(),
    //     )]);
    //     let client =
    //         Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
    //     let store = DynamoDBStore::new(client, "test".to_string());

    //     // WHEN getting an item
    //     let res = store.get("1").await?;

    //     // THEN the response has the correct values
    //     if let Some(product) = res {
    //         assert_eq!(product.id, "1");
    //         assert_eq!(product.name, "test1");
    //         assert_eq!(product.price, 1.0);
    //     } else {
    //         panic!("Expected product to be Some");
    //     }
    //     // AND the request matches the expected request
    //     conn.assert_requests_match(&vec![]);

    //     Ok(())
    // }

    // #[tokio::test]
    // async fn test_put() -> Result<(), Error> {
    //     // GIVEN an empty DynamoDBStore and a product
    //     let conn = TestConnection::new(vec![(
    //         get_request_builder()
    //             .header("x-amz-target", "DynamoDB_20120810.PutItem")
    //             .body(SdkBody::from(r#"{"TableName":"test","Item":{"id":{"S":"1"},"name":{"S":"test1"},"price":{"N":"1.5"}}}"#))
    //             .unwrap(),
    //         http::Response::builder()
    //             .status(200)
    //             .body(SdkBody::from(r#"{"Attributes": {"id": {"S": "1"}, "name": {"S": "test1"}, "price": {"N": "1.5"}}}"#))
    //             .unwrap(),
    //     )]);
    //     let client =
    //         Client::from_conf_conn(get_mock_config().await, DynConnector::new(conn.clone()));
    //     let store = DynamoDBStore::new(client, "test".to_string());
    //     let product = Product {
    //         id: "1".to_string(),
    //         name: "test1".to_string(),
    //         price: 1.5,
    //     };

    //     // WHEN putting an item
    //     store.put(&product).await?;

    //     // THEN the request matches the expected request
    //     conn.assert_requests_match(&vec![]);

    //     Ok(())
    // }

    // #[test]
    // fn product_from_dynamodb() {
    //     let mut value = HashMap::new();
    //     value.insert("id".to_owned(), AttributeValue::S("id".to_owned()));
    //     value.insert("name".to_owned(), AttributeValue::S("name".to_owned()));
    //     value.insert("price".to_owned(), AttributeValue::N("1.0".to_owned()));

    //     let test_run = TestRun::try_from(value).unwrap();
    //     assert_eq!(test_run.id, "id");
    //     assert_eq!(test_run.language, "language");
    //     assert_eq!(test_run.payload, "name");
    //     assert_eq!(test_run.status, );
    // }

    // #[test]
    // fn product_to_dynamodb() -> Result<(), Error> {
    //     let product = Product {
    //         id: "id".to_owned(),
    //         name: "name".to_owned(),
    //         price: 1.5,
    //     };

    //     let value: HashMap<String, AttributeValue> = (&product).into();
    //     assert_eq!(value.get("id").unwrap().as_s().unwrap(), "id");
    //     assert_eq!(value.get("name").unwrap().as_s().unwrap(), "name");
    //     assert_eq!(value.get("price").unwrap().as_n().unwrap(), "1.5");

    //     Ok(())
    // }
}