use axum::{response::Html, response::IntoResponse};

/// Serve the playground shell HTML.
pub(super) async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

/// Serve the playground browser application JavaScript.
pub(super) async fn js_handler() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../../static/app.js"),
    )
}

/// Serve the playground stylesheet.
pub(super) async fn css_handler() -> impl IntoResponse {
    (
        [("Content-Type", "text/css")],
        include_str!("../../static/style.css"),
    )
}
