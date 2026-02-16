use crate::{
    db::users as db_users,
    error::AppError,
    middleware::auth::{create_access_token, create_refresh_token, hash_token, verify_access_token, AuthUser},
    models::user::*,
    routes::documents::AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use serde_json::{json, Value};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate input
    if req.username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }
    if !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email address".to_string()));
    }

    // Check if username already exists
    if db_users::find_by_username(&state.pool, &req.username).await?.is_some() {
        return Err(AppError::Conflict("Username already exists".to_string()));
    }

    // Check if email already exists
    if db_users::find_by_email(&state.pool, &req.email).await?.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // Hash password with Argon2id
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();

    // Create user
    let user_id = uuid::Uuid::now_v7().to_string();
    let user = db_users::create_user(&state.pool, &user_id, &req.username, &req.email, &password_hash).await?;

    // Generate tokens
    let access_token = create_access_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;
    let refresh_token = create_refresh_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    // Store refresh token hash
    let token_id = uuid::Uuid::now_v7().to_string();
    let token_hash = hash_token(&refresh_token);
    let expires_at = (Utc::now() + Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    db_users::store_refresh_token(&state.pool, &token_id, &user.id, &token_hash, &expires_at).await?;

    Ok(Json(AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Find user by username
    let user = db_users::find_by_username(&state.pool, &req.username)
        .await?
        .ok_or(AppError::Unauthorized("Invalid username or password".to_string()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Password hash parse error: {}", e)))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid username or password".to_string()))?;

    // Generate tokens
    let access_token = create_access_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;
    let refresh_token = create_refresh_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    // Store refresh token hash
    let token_id = uuid::Uuid::now_v7().to_string();
    let token_hash = hash_token(&refresh_token);
    let expires_at = (Utc::now() + Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    db_users::store_refresh_token(&state.pool, &token_id, &user.id, &token_hash, &expires_at).await?;

    Ok(Json(AuthResponse {
        user: user.into(),
        access_token,
        refresh_token,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Verify the refresh token JWT
    let _claims = verify_access_token(&req.refresh_token, &state.jwt_secret)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))?;

    // Check if refresh token hash exists in DB
    let token_hash = hash_token(&req.refresh_token);
    let (_token_id, user_id, expires_at) = db_users::find_refresh_token(&state.pool, &token_hash)
        .await?
        .ok_or(AppError::Unauthorized("Refresh token not found or revoked".to_string()))?;

    // Check expiration
    let expires = chrono::NaiveDateTime::parse_from_str(&expires_at, "%Y-%m-%dT%H:%M:%S%.3fZ")
        .map_err(|e| AppError::Internal(format!("Date parse error: {}", e)))?;
    if expires.and_utc() < Utc::now() {
        // Delete expired token
        db_users::delete_refresh_token(&state.pool, &token_hash).await?;
        return Err(AppError::Unauthorized("Refresh token expired".to_string()));
    }

    // Verify user still exists
    let user = db_users::find_by_id(&state.pool, &user_id)
        .await?
        .ok_or(AppError::Unauthorized("User not found".to_string()))?;

    // Delete old refresh token
    db_users::delete_refresh_token(&state.pool, &token_hash).await?;

    // Generate new tokens
    let new_access_token = create_access_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;
    let new_refresh_token = create_refresh_token(&user.id, &state.jwt_secret)
        .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))?;

    // Store new refresh token hash
    let new_token_id = uuid::Uuid::now_v7().to_string();
    let new_token_hash = hash_token(&new_refresh_token);
    let new_expires_at = (Utc::now() + Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    db_users::store_refresh_token(&state.pool, &new_token_id, &user.id, &new_token_hash, &new_expires_at).await?;

    Ok(Json(AuthResponse {
        user: user.into(),
        access_token: new_access_token,
        refresh_token: new_refresh_token,
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Value>, AppError> {
    // Delete all refresh tokens for this user
    db_users::delete_user_refresh_tokens(&state.pool, &auth_user.user_id).await?;

    Ok(Json(json!({ "message": "Logged out successfully" })))
}

pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = db_users::find_by_id(&state.pool, &auth_user.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user.into()))
}
