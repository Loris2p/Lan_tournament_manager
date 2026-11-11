use crate::errors::Errors;
use crate::models::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::broadcast::error::RecvError;
const MAX_WS_CONNECTIONS: usize = 150;
struct WsGuard(Arc<AppState>);
impl Drop for WsGuard {
    fn drop(&mut self) {
        self.0.active_ws_connections.fetch_sub(1, Ordering::SeqCst);
    }
}
pub async fn ws_handler(
    ws_handler: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Errors> {
    if !is_same_origin(&headers)? {
        return Err(Errors::Error403Forbidden(
            "Origine non autorisée.".to_string(),
        ));
    }
    if state.active_ws_connections.load(Ordering::Relaxed) >= MAX_WS_CONNECTIONS {
        return Err(Errors::Error429TooManyRequests(
            "Trop de connexions en simultané".to_string(),
        ));
    }
    Ok(ws_handler.on_upgrade(move |socket| handle_socket(socket, state)))
}
pub async fn handle_socket(mut ws: WebSocket, state: Arc<AppState>) {
    state.active_ws_connections.fetch_add(1, Ordering::SeqCst);
    let _guard = WsGuard(state.clone());
    let mut rx = state.ws_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(m) => {
                let transformed_msg = Message::text(m);
                if ws.send(transformed_msg).await.is_err() {
                    break;
                }
            }
            Err(RecvError::Lagged(_)) => continue,
            Err(RecvError::Closed) => break,
        }
    }
}
fn is_same_origin(headers: &HeaderMap) -> Result<bool, Errors> {
    let expected_host = match headers
        .get("x-forwarded-host")
        .or_else(|| headers.get("host"))
        .and_then(|val| val.to_str().ok())
        .map(|h| h.trim())
    {
        Some(s) => s,
        None => {
            return Err(Errors::Error400BadRequest(
                "Un problème est survenu.".to_string(),
            ));
        }
    };
    let origin_host = match headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(|o| {
            o.trim_start_matches("https://")
                .trim_start_matches("http://")
        })
        .map(|o| o.trim_end_matches('/'))
    {
        Some(s) => s,
        None => {
            return Err(Errors::Error400BadRequest(
                "Un problème est survenu.".to_string(),
            ));
        }
    };

    Ok(expected_host == origin_host)
}
