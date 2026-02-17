use crate::error::AppError;
use crate::models::user::User;
use sqlx::SqlitePool;

pub async fn create_user(
    pool: &SqlitePool,
    id: &str,
    username: &str,
    email: Option<&str>,
    password_hash: &str,
) -> Result<User, AppError> {
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .execute(pool)
    .await?;

    find_by_id(pool, id)
        .await?
        .ok_or(AppError::Internal("Failed to retrieve created user".to_string()))
}

pub async fn find_by_username(pool: &SqlitePool, username: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, created_at, updated_at
        FROM users
        WHERE username = ?
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn find_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, created_at, updated_at
        FROM users
        WHERE email = ?
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn store_refresh_token(
    pool: &SqlitePool,
    id: &str,
    user_id: &str,
    token_hash: &str,
    expires_at: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_refresh_token(
    pool: &SqlitePool,
    token_hash: &str,
) -> Result<Option<(String, String, String)>, AppError> {
    let row = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT id, user_id, expires_at
        FROM refresh_tokens
        WHERE token_hash = ?
        "#,
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete_refresh_token(pool: &SqlitePool, token_hash: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = ?")
        .bind(token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_user_refresh_tokens(pool: &SqlitePool, user_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
