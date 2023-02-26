use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Test {
    pub message: String,
    pub name: String,
    pub status: String,

    pub actual_output: String,
    pub expected_output: String,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TestRun {
    pub id: String,
    pub files: HashMap<String, String>,
    pub language: String,
    pub status: String,
    pub tests: Vec<Test>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    Created { testrun: TestRun },
    Updated { old: TestRun, new: TestRun },
    Deleted { testrun: TestRun },
}

impl Event {
    pub fn id(&self) -> &str {
        match self {
            Event::Created { testrun } => testrun.id.as_str(),
            Event::Updated { new, .. } => new.id.as_str(),
            Event::Deleted { testrun } => testrun.id.as_str(),
        }
    }
}
