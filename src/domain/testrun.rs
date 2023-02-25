pub async fn get_test_run(store: &dyn StoreGet, id: &str) -> Result<Option<TestRun>, Error> {
    store.get(id).await
}

pub async fn put_test_run(store: &dyn StorePut, test_run: &TestRun) -> Result<(), Error> {
    store.put(test_run).await
}

pub async fn delete_test_run(store: &dyn StoreDelete, id: &str) -> Result<(), Error> {
    store.delete(id).await
}