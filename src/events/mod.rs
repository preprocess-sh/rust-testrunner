use async_trait::async_trait;

use crate::error::Error;

mod eventbridge;

#[async_trait]
pub trait EventBus {
    type E;

    async fn send_event(&self, event: &Self::E) -> Result<(), Error>;
    async fn send_events(&self, events: &[Self::E]) -> Result<(), Error>;
}
