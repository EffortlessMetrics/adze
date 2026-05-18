use crate::PlaygroundSession;
use anyhow::Result;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use super::{
    assets::{css_handler, index_handler, js_handler},
    handlers::{
        analyze_handler, export_handler, import_handler, parse_handler, test_handler, tests_handler,
    },
    state::shared_session,
};

/// Launch the web server for the playground.
pub fn launch_server(session: PlaygroundSession, port: u16) -> Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let app = build_router(session);
        let addr = format!("0.0.0.0:{}", port).parse().unwrap();
        println!("🚀 Playground server running at http://localhost:{}", port);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}

fn build_router(session: PlaygroundSession) -> Router {
    Router::new()
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
        .with_state(shared_session(session))
}
