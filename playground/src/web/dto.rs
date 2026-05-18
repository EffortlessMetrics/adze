use serde::{Deserialize, Serialize};

/// JSON request payload for parse operations.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ParseRequest {
    pub(super) input: String,
    pub(super) visualize: Option<bool>,
}

/// JSON request payload used to add a playground test case.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct TestRequest {
    pub(super) name: String,
    pub(super) input: String,
    pub(super) expected: Option<String>,
    pub(super) tags: Vec<String>,
}

/// Query-string options shared by web export routes.
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct ExportQueryParams {
    pub(super) format: Option<String>,
}
