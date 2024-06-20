use axum::{Extension, Json};
use axum::extract::Query;
use axum_garde::WithValidation;
use sqlx::{Pool, Postgres};

use crate::app::auth::AuthSession;
use crate::app::server::AppState;
use crate::json::error::ApiError;
use crate::json::utils::Pagination;
use crate::models::products::SellerProductPreview;

pub async fn more_orders(
    Extension(state): Extension<AppState>,
    WithValidation(query): WithValidation<Query<Pagination>>,
    auth_session: AuthSession,
) -> Result<Json<Vec<SellerProductPreview>>, ApiError> {
    let user = auth_session.user.unwrap();

    Ok(Json(fetch(user.id, query.into_inner(), &state.pool).await?))
}

pub async fn fetch(
    id: i32,
    query: Pagination,
    pool: &Pool<Postgres>,
) -> Result<Vec<SellerProductPreview>, ApiError> {
    let more_orders = sqlx::query_as::<_, SellerProductPreview>(
        r#"
            SELECT sp.id, p.id AS product_id, p.name,
                sp.photos, sp.price,
                COALESCE(sp.rating_sum / NULLIF(sp.rating_quantity, 0), NULL) AS rating,
                sp.rating_quantity
            FROM cart c
            LEFT JOIN seller_products sp ON c.seller_product_id = sp.id
            JOIN products p ON sp.product_id = p.id
            WHERE status = 2
            GROUP BY sp.id, p.id
            ORDER BY COUNT(*) DESC
            LIMIT $2 OFFSET $3
        "#
    )
        .bind(id)
        .bind(query.per_page)
        .bind((query.page - 1) * query.per_page)
        .fetch_all(pool)
        .await?;

    Ok(more_orders)
}