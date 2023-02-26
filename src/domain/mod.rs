use crate::{error::Error, events::EventBus, model::Event};

pub mod testrun;

pub async fn send_events(
    event_bus: &dyn EventBus<E = Event>,
    events: &[Event],
) -> Result<(), Error> {
    event_bus.send_events(events).await
}
