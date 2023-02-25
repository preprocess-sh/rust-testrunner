use aws_sdk_eventbridge::model::PutEventsRequestEntry;

use crate::model::Event;

static SOURCE: &str = "preprocess-test-runs";

pub trait EventExt {
    fn to_eventbridge(&self, bus_name: &str) -> PutEventsRequestEntry;
}

impl EventExt for Event {
    fn to_eventbridge(&self, bus_name: &str) -> PutEventsRequestEntry {
        PutEventsRequestEntry::builder()
            .event_bus_name(bus_name)
            .source(SOURCE)
            .detail_type(match self {
                Event::Created { .. } => "TestRunCreated",
                Event::Updated { .. } => "TestRunUpdated",
                Event::Deleted { .. } => "TestRunDeleted",
            })
            .resources(self.id())
            .detail(serde_json::to_string(self).unwrap())
            .build()
    }
}
