use crate::{error::Error, model::TestRun};
use async_trait::async_trait;

mod dynamodb;

pub use dynamodb::DynamoDBStore;

pub trait Store: StoreGet + StorePut + StoreDelete {}

/// Trait for retrieving a single testrun
#[async_trait]
pub trait StoreGet: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<TestRun>, Error>;
}

/// Trait for storing a single testrun
#[async_trait]
pub trait StorePut: Send + Sync {
    async fn put(&self, testrun: &TestRun) -> Result<(), Error>;
}

/// Trait for deleting a single testrun
#[async_trait]
pub trait StoreDelete: Send + Sync {
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
