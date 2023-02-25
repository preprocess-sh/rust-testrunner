pub async fn get_test_run(store: &dyn StoreGet, id: &str) -> Result<Option<TestRun>, Error> {
    store.get(id).await
}