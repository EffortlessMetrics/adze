use crate::TestCase;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

use super::{
    dto::{ExportQueryParams, ParseRequest, TestRequest},
    state::SharedSession,
};

/// Parse input text using the current playground session.
pub(super) async fn parse_handler(
    State(session): State<SharedSession>,
    Json(req): Json<ParseRequest>,
) -> Response {
    let session = session.lock().await;

    match session.parse(&req.input) {
        Ok(mut result) => {
            if req.visualize.unwrap_or(false)
                && let Some(tree) = &result.tree
            {
                result.visualization = session.visualize_tree(tree).ok();
            }
            Json(result).into_response()
        }
        Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

/// Add a new test case to the current playground session.
pub(super) async fn test_handler(
    State(session): State<SharedSession>,
    Json(req): Json<TestRequest>,
) -> impl IntoResponse {
    let mut session = session.lock().await;

    session.add_test_case(TestCase {
        name: req.name,
        input: req.input,
        expected_tree: req.expected,
        should_pass: true,
        tags: req.tags,
    });

    Json(serde_json::json!({ "success": true }))
}

/// Run all test cases in the current playground session.
pub(super) async fn tests_handler(State(session): State<SharedSession>) -> impl IntoResponse {
    let session = session.lock().await;
    let results = session.run_tests();
    Json(results)
}

/// Analyze the current session grammar.
pub(super) async fn analyze_handler(State(session): State<SharedSession>) -> Response {
    let mut session = session.lock().await;

    match session.analyze_grammar() {
        Ok(analysis) => Json(serde_json::to_value(analysis).unwrap()).into_response(),
        Err(e) => json_error(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

/// Export the current playground session.
pub(super) async fn export_handler(
    State(session): State<SharedSession>,
    Query(params): Query<ExportQueryParams>,
) -> impl IntoResponse {
    let session = session.lock().await;

    match session.export() {
        Ok(data) => {
            if params.format.as_deref() == Some("download") {
                (
                    [(
                        "Content-Disposition",
                        "attachment; filename=\"playground-session.json\"",
                    )],
                    data,
                )
                    .into_response()
            } else {
                data.into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// Import a serialized playground session.
pub(super) async fn import_handler(State(session): State<SharedSession>, body: String) -> Response {
    let mut session = session.lock().await;

    match session.import(&body) {
        Ok(()) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(e) => json_error(StatusCode::BAD_REQUEST, e.to_string()),
    }
}

fn json_error(status: StatusCode, message: String) -> Response {
    let error_response = serde_json::json!({
        "error": message
    });
    (status, Json(error_response)).into_response()
}
