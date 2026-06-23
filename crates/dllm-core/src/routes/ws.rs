//! WebSocket 路由

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Extension,
    response::Response,
};
use std::sync::Arc;

use crate::engine_pool::EnginePool;

pub async fn monitor_handler(
    ws: WebSocketUpgrade,
    Extension(pool): Extension<Arc<EnginePool>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, pool))
}

async fn handle_socket(mut socket: WebSocket, pool: Arc<EnginePool>) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    // TODO: 處理監控訂閱請求
                    let _ = socket
                        .send(Message::Text(format!("收到: {}", text)))
                        .await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    }
}
