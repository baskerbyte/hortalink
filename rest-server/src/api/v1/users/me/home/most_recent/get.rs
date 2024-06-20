use axum::{Extension, Json};
use axum::extract::Query;
use axum_garde::WithValidation;
use sqlx::{Pool, Postgres};

use crate::app::auth::AuthSession;
use crate::app::server::AppState;
use crate::json::error::ApiError;
use crate::json::utils::Pagination;
use crate::models::products::SellerProductPreview;

pub async fn viewed_recently(
    Extension(state): Extension<AppState>,
    auth_session: AuthSession,
    WithValidation(query): WithValidation<Query<Pagination>>,
) -> Result<Json<Vec<SellerProductPreview>>, ApiError> {
    let user = auth_session.user.unwrap();

    Ok(Json(fetch(user.id, query.into_inner(), &state.pool).await?))
}

pub async fn fetch(
    id: i32,
    query: Pagination,
    pool: &Pool<Postgres>,
) -> Result<Vec<SellerProductPreview>, ApiError> {
    let seen_recently = sqlx::query_as::<_, SellerProductPreview>(
        r#"
                SELECT sp.id, p.id AS product_id, p.name,
                    sp.photos, sp.price,
                    COALESCE(sp.rating_sum / NULLIF(sp.rating_quantity, 0), NULL) AS rating,
                    sp.rating_quantity
                FROM products_seen_recently sr
                LEFT JOIN seller_products sp ON sp.id = sr.seller_product_id
                JOIN products p ON sp.product_id = p.id
                WHERE sr.customer = $1
                ORDER BY sr.viewed_at DESC
                LIMIT $2 OFFSET $3
            "#
    )
        .bind(id)
        .bind(query.per_page)
        .bind((query.page - 1) * query.per_page)
        .fetch_all(pool)
        .await?;

    Ok(seen_recently)
}