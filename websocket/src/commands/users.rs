use std::net::SocketAddr;

use sqlx::{Pool, Postgres};

use crate::json::user::Record;
use crate::server::session::SocketSession;

pub fn disconnect(addr: SocketAddr, connections: &mut Vec<SocketSession>) {
    let idx = match connections.iter().position(|session| session.addr == addr) {
        None => return,
        Some(value) => value
    };

    connections.remove(idx);
}

pub async fn identify(
    (addr, session_id): (SocketAddr, String),
    connections: &mut Vec<SocketSession>,
    pool: &Pool<Postgres>,
) {
    let client = match connections.iter_mut().find(|session| session.addr == addr) {
        None => return,
        Some(value) => value
    };

    let session_data: Option<Vec<u8>> = sqlx::query_scalar(
        r#"
            SELECT data FROM "tower_sessions"."sessions"
            WHERE id = $1
        "#
    )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .unwrap();

    if let Some(data) = session_data {
        let user_id = rmp_serde::from_slice::<Record>(&data).unwrap()
            .data["axum-login.data"]["user_id"].as_i64().unwrap();

        client.user_id = Some(user_id as i32);
    }
}