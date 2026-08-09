use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    error::{AppError, AppResult},
    state::AppState,
};

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ArticleRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub excerpt: Option<String>,
    pub content_md: String,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub author_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_user_id: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub og_image_url: Option<String>,
    pub published_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticleRequest {
    pub title: String,
    #[serde(alias = "content_md")]
    pub content_md: String,
    pub excerpt: Option<String>,
    #[serde(alias = "cover_image_url")]
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    #[serde(alias = "seo_title")]
    pub seo_title: Option<String>,
    #[serde(alias = "seo_description")]
    pub seo_description: Option<String>,
    #[serde(alias = "og_image_url")]
    pub og_image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticleRequest {
    pub title: Option<String>,
    #[serde(alias = "content_md")]
    pub content_md: Option<String>,
    pub excerpt: Option<String>,
    #[serde(alias = "cover_image_url")]
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    #[serde(alias = "seo_title")]
    pub seo_title: Option<String>,
    #[serde(alias = "seo_description")]
    pub seo_description: Option<String>,
    #[serde(alias = "og_image_url")]
    pub og_image_url: Option<String>,
}

fn normalize_slug(title: &str, existing_slug: &str) -> String {
    let base = if !existing_slug.trim().is_empty() {
        existing_slug.trim().to_string()
    } else {
        slug::slugify(title)
    };
    if base.is_empty() {
        "untitled".to_string()
    } else {
        base
    }
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Public: list published articles, newest first.
pub async fn list_published(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ArticleRow>>> {
    let rows = sqlx::query_as::<_, ArticleRow>(
        "SELECT id, slug, title, excerpt, content_md, cover_image_url, tags, status,
                author_name, author_user_id, seo_title, seo_description, og_image_url,
                published_at, created_at, updated_at
         FROM articles WHERE status = 'published'
         ORDER BY published_at DESC NULLS LAST, created_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(rows))
}

/// Public: fetch one published article by slug.
pub async fn get_published_by_slug(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> AppResult<Json<ArticleRow>> {
    let row = sqlx::query_as::<_, ArticleRow>(
        "SELECT id, slug, title, excerpt, content_md, cover_image_url, tags, status,
                author_name, author_user_id, seo_title, seo_description, og_image_url,
                published_at, created_at, updated_at
         FROM articles WHERE slug = $1 AND status = 'published'",
    )
    .bind(slug)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    .ok_or_else(|| AppError::NotFound("Article not found".into()))?;
    Ok(Json(row))
}

/// Admin: list all articles (drafts + published), newest first.
pub async fn list_all(
    State(state): State<AppState>,
    _user: CurrentUser,
) -> AppResult<Json<Vec<ArticleRow>>> {
    let rows = sqlx::query_as::<_, ArticleRow>(
        "SELECT id, slug, title, excerpt, content_md, cover_image_url, tags, status,
                author_name, author_user_id, seo_title, seo_description, og_image_url,
                published_at, created_at, updated_at
         FROM articles ORDER BY updated_at DESC",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(rows))
}

/// Admin: create an article.
pub async fn create_article(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(req): Json<CreateArticleRequest>,
) -> AppResult<(StatusCode, Json<ArticleRow>)> {
    if state.config.admin_only && !user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can create articles".into(),
        ));
    }
    crate::error::require_non_blank(&[("title", &req.title), ("content_md", &req.content_md)])?;

    let status = req
        .status
        .unwrap_or_else(|| "draft".into())
        .to_lowercase();
    if !["draft", "published"].contains(&status.as_str()) {
        return Err(AppError::BadRequest("status must be draft or published".into()));
    }

    let slug = normalize_slug(&req.title, "");
    let tags = req.tags.unwrap_or_default();
    let author_name = user.username.clone();

    let row = sqlx::query_as::<_, ArticleRow>(
        "INSERT INTO articles
            (slug, title, excerpt, content_md, cover_image_url, tags, status,
             author_name, author_user_id, seo_title, seo_description, og_image_url, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                 CASE WHEN $7 = 'published' THEN now() ELSE NULL END)
         RETURNING id, slug, title, excerpt, content_md, cover_image_url, tags, status,
                   author_name, author_user_id, seo_title, seo_description, og_image_url,
                   published_at, created_at, updated_at",
    )
    .bind(slug)
    .bind(req.title.trim())
    .bind(req.excerpt.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.content_md.trim())
    .bind(req.cover_image_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(&tags)
    .bind(&status)
    .bind(&author_name)
    .bind(&user.user_id)
    .bind(req.seo_title.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.seo_description.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.og_image_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if is_unique_violation(&e) {
            AppError::Conflict("An article with this slug already exists".into())
        } else {
            AppError::Internal(anyhow::anyhow!(e))
        }
    })?;

    Ok((StatusCode::CREATED, Json(row)))
}

/// Admin: update an article by id.
pub async fn update_article(
    State(state): State<AppState>,
    user: CurrentUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(req): Json<UpdateArticleRequest>,
) -> AppResult<Json<ArticleRow>> {
    if state.config.admin_only && !user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can edit articles".into(),
        ));
    }

    let existing_status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM articles WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    .ok_or_else(|| AppError::NotFound("Article not found".into()))?;

    let status = req
        .status
        .map(|s| s.to_lowercase())
        .unwrap_or(existing_status);
    if !["draft", "published"].contains(&status.as_str()) {
        return Err(AppError::BadRequest("status must be draft or published".into()));
    }

    let row = sqlx::query_as::<_, ArticleRow>(
        "UPDATE articles SET
            title = COALESCE($2, title),
            excerpt = COALESCE($3, excerpt),
            content_md = COALESCE($4, content_md),
            cover_image_url = COALESCE($5, cover_image_url),
            tags = COALESCE($6, tags),
            seo_title = COALESCE($7, seo_title),
            seo_description = COALESCE($8, seo_description),
            og_image_url = COALESCE($9, og_image_url),
            status = $10,
            published_at = CASE
                WHEN $10 = 'published' AND status = 'draft' THEN now()
                WHEN $10 = 'draft' THEN NULL
                ELSE published_at
            END,
            updated_at = now()
         WHERE id = $1
         RETURNING id, slug, title, excerpt, content_md, cover_image_url, tags, status,
                   author_name, author_user_id, seo_title, seo_description, og_image_url,
                   published_at, created_at, updated_at",
    )
    .bind(id)
    .bind(req.title.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.excerpt.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.content_md.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.cover_image_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(&req.tags)
    .bind(req.seo_title.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.seo_description.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(req.og_image_url.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    .bind(&status)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(row))
}

/// Admin: delete an article by id.
pub async fn delete_article(
    State(state): State<AppState>,
    user: CurrentUser,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    if state.config.admin_only && !user.is_admin() {
        return Err(AppError::Forbidden(
            "Only administrators can delete articles".into(),
        ));
    }
    let deleted = sqlx::query("DELETE FROM articles WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::NotFound("Article not found".into()));
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db) if db.is_unique_violation())
}
