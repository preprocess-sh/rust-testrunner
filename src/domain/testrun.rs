use crate::{
    error::Error,
    model::TestRun,
    store::{StoreDelete, StoreGet, StorePut},
};

pub async fn get_testrun(store: &dyn StoreGet, id: &str) -> Result<Option<TestRun>, Error> {
    store.get(id).await
}

pub async fn put_testrun(store: &dyn StorePut, testrun: &TestRun) -> Result<(), Error> {
    store.put(testrun).await
}

pub async fn delete_testrun(store: &dyn StoreDelete, id: &str) -> Result<(), Error> {
    store.delete(id).await
}
