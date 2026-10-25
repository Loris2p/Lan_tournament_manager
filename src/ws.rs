use crate::models::AppState;
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use std::sync::Arc;

pub async fn ws_handler(ws_handler: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws_handler.on_upgrade(move |socket| handle_socket(socket, state))
}
pub async fn handle_socket(mut ws: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        let transformed_msg = Message::text(msg);
        if ws.send(transformed_msg).await.is_err() {
            break;
        }
    }
}
