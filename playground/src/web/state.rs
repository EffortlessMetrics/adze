use crate::PlaygroundSession;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared, mutex-protected playground session state used by all web handlers.
pub(super) type SharedSession = Arc<Mutex<PlaygroundSession>>;

/// Wrap a playground session in the shared state container expected by Axum.
pub(super) fn shared_session(session: PlaygroundSession) -> SharedSession {
    Arc::new(Mutex::new(session))
}
