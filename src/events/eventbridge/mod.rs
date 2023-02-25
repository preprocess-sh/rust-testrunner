use async_trait::async_trait;
use aws_sdk_eventbridge::Client;
use futures::future::join_all;
use tracing::instrument;

use crate::{error::Error, model::Event};

use self::ext::EventExt;

use super::EventBus;

mod ext;

pub struct EventBridgeBus {
    client: Client,
    bus_name: String,
}

impl EventBridgeBus {
    pub fn new(client: Client, bus_name: String) -> Self {
        Self { client, bus_name }
    }
}

#[async_trait]
impl EventBus for EventBridgeBus {
    type E = Event;

    #[instrument(skip(self))]
    async fn send_event(&self, event: &Self::E) -> Result<(), Error> {
        self.client
            .put_events()
            .entries(event.to_eventbridge(&self.bus_name))
            .send()
            .await?;

        Ok(())
    }

    #[instrument(skip(self, events))]
    async fn send_events(&self, events: &[Self::E]) -> Result<(), Error> {
        let response = join_all(events.iter().collect::<Vec<_>>().chunks(10).map(|chunk| {
            self.client
                .put_events()
                .set_entries(Some(
                    chunk
                        .iter()
                        .map(|e| e.to_eventbridge(&self.bus_name))
                        .collect::<Vec<_>>(),
                ))
                .send()
        }))
        .await;

        response.into_iter().collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }
}
