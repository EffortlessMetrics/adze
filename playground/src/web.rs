// Web interface for the rust-sitter playground

use crate::PlaygroundSession;
use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

type SharedSession = Arc<Mutex<PlaygroundSession>>;

#[derive(Debug, Serialize, Deserialize)]
struct ParseRequest {
    input: String,
    visualize: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestRequest {
    name: String,
    input: String,
    expected: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryParams {
    format: Option<String>,
}

/// Launch the web server for the playground
pub fn launch_server(session: PlaygroundSession, port: u16) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let shared_session = Arc::new(Mutex::new(session));
        
        let app = Router::new()
            .route("/", get(index_handler))
            .route("/api/parse", post(parse_handler))
            .route("/api/test", post(test_handler))
            .route("/api/tests", get(tests_handler))
            .route("/api/analyze", get(analyze_handler))
            .route("/api/export", get(export_handler))
            .route("/api/import", post(import_handler))
            .route("/static/app.js", get(js_handler))
            .route("/static/style.css", get(css_handler))
            .layer(CorsLayer::permissive())
            .with_state(shared_session);

        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        println!("🚀 Playground server running at http://localhost:{}", port);
        
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn parse_handler(
    State(session): State<SharedSession>,
    Json(req): Json<ParseRequest>,
) -> Response {
    let session = session.lock().await;
    
    match session.parse(&req.input) {
        Ok(mut result) => {
            if req.visualize.unwrap_or(false) {
                if let Some(tree) = &result.tree {
                    result.visualization = session.visualize_tree(tree).ok();
                }
            }
            Json(result).into_response()
        }
        Err(e) => {
            let error_response = serde_json::json!({
                "error": e.to_string()
            });
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}

async fn test_handler(
    State(session): State<SharedSession>,
    Json(req): Json<TestRequest>,
) -> impl IntoResponse {
    let mut session = session.lock().await;
    
    session.add_test_case(crate::TestCase {
        name: req.name,
        input: req.input,
        expected_tree: req.expected,
        should_pass: true,
        tags: req.tags,
    });
    
    Json(serde_json::json!({ "success": true }))
}

async fn tests_handler(
    State(session): State<SharedSession>,
) -> impl IntoResponse {
    let session = session.lock().await;
    let results = session.run_tests();
    Json(results)
}

async fn analyze_handler(
    State(session): State<SharedSession>,
) -> Response {
    let mut session = session.lock().await;
    
    match session.analyze_grammar() {
        Ok(analysis) => Json(serde_json::to_value(analysis).unwrap()).into_response(),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": e.to_string()
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

async fn export_handler(
    State(session): State<SharedSession>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let session = session.lock().await;
    
    match session.export() {
        Ok(data) => {
            if params.format.as_deref() == Some("download") {
                ([("Content-Disposition", "attachment; filename=\"playground-session.json\"")], data).into_response()
            } else {
                data.into_response()
            }
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn import_handler(
    State(session): State<SharedSession>,
    body: String,
) -> Response {
    let mut session = session.lock().await;
    
    match session.import(&body) {
        Ok(_) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(e) => {
            let error_response = serde_json::json!({
                "error": e.to_string()
            });
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}

async fn js_handler() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../static/app.js"),
    )
}

async fn css_handler() -> impl IntoResponse {
    (
        [("Content-Type", "text/css")],
        include_str!("../static/style.css"),
    )
}