use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::{extract_bearer, generate_api_key, hash_password, sign_jwt, verify_jwt, verify_password};
use crate::db::{ChatMessage, ChatSession, ChatSource, Database};
use crate::models::*;
use crate::rag::RagEngine;

// ── App state yang di-share ke semua handler ──
pub struct AppState {
    pub db:         Arc<Database>,
    pub rag:        Arc<RagEngine>,
    pub jwt_secret: String,
}

// ══════════════════════════════════════════════════════════════════
//  MIDDLEWARE HELPER — ekstrak user dari JWT
// ══════════════════════════════════════════════════════════════════

fn require_auth(req: &HttpRequest, jwt_secret: &str) -> Result<JwtClaims, HttpResponse> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ApiError::new(401, "Token tidak ditemukan"))
        })?;

    let token = extract_bearer(auth).ok_or_else(|| {
        HttpResponse::Unauthorized().json(ApiError::new(401, "Format token tidak valid"))
    })?;

    verify_jwt(token, jwt_secret).map_err(|_| {
        HttpResponse::Unauthorized().json(ApiError::new(401, "Token tidak valid atau sudah expired"))
    })
}

// ══════════════════════════════════════════════════════════════════
//  AUTH HANDLERS
// ══════════════════════════════════════════════════════════════════

pub async fn register(
    state: web::Data<AppState>,
    body:  web::Json<RegisterRequest>,
) -> HttpResponse {
    if body.email.is_empty() || body.password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Email tidak valid atau password kurang dari 8 karakter"));
    }

    let hash = match tokio::task::spawn_blocking({
        let pw = body.password.clone();
        move || hash_password(&pw)
    })
    .await
    {
        Ok(Ok(h)) => h,
        _ => return HttpResponse::InternalServerError()
                .json(ApiError::new(500, "Gagal memproses password")),
    };

    let display = body.display_name.clone().unwrap_or_default();
    match state.db.create_user(&body.email, &hash, &display).await {
        Ok(user) => {
            let (token, expires_in) =
                match sign_jwt(user.id, &user.email, &user.role, &state.jwt_secret) {
                    Ok(t) => t,
                    Err(_) => return HttpResponse::InternalServerError()
                        .json(ApiError::new(500, "Gagal membuat token")),
                };
            HttpResponse::Created().json(ApiSuccess::new(AuthResponse {
                token,
                token_type: "Bearer".to_string(),
                expires_in,
                user: UserPublic {
                    id:           user.id,
                    email:        user.email,
                    display_name: user.display_name,
                    role:         user.role,
                    created_at:   user.created_at,
                },
            }))
        }
        Err(e) => HttpResponse::Conflict().json(ApiError::new(409, e)),
    }
}

pub async fn login(
    state: web::Data<AppState>,
    body:  web::Json<LoginRequest>,
) -> HttpResponse {
    let user = match state.db.get_user_by_email(&body.email).await {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized()
            .json(ApiError::new(401, "Email atau password salah")),
    };

    if !user.is_active {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Akun tidak aktif"));
    }

    let hash = user.password_hash.clone();
    let pw   = body.password.clone();
    let ok   = tokio::task::spawn_blocking(move || verify_password(&pw, &hash))
        .await
        .unwrap_or(Ok(false))
        .unwrap_or(false);

    if !ok {
        return HttpResponse::Unauthorized()
            .json(ApiError::new(401, "Email atau password salah"));
    }

    let (token, expires_in) =
        match sign_jwt(user.id, &user.email, &user.role, &state.jwt_secret) {
            Ok(t) => t,
            Err(_) => return HttpResponse::InternalServerError()
                .json(ApiError::new(500, "Gagal membuat token")),
        };

    HttpResponse::Ok().json(ApiSuccess::new(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserPublic {
            id:           user.id,
            email:        user.email,
            display_name: user.display_name,
            role:         user.role,
            created_at:   user.created_at,
        },
    }))
}

pub async fn get_me(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.get_user_by_id(uid).await {
        Ok(Some(u)) => HttpResponse::Ok().json(ApiSuccess::new(UserPublic {
            id:           u.id,
            email:        u.email,
            display_name: u.display_name,
            role:         u.role,
            created_at:   u.created_at,
        })),
        _ => HttpResponse::NotFound().json(ApiError::new(404, "User tidak ditemukan")),
    }
}

pub async fn update_profile(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.update_user_profile(
        uid,
        body.display_name.as_deref(),
        body.email.as_deref(),
    ).await {
        Ok(_)  => HttpResponse::Ok().json(ApiSuccess::new("Profil berhasil diperbarui")),
        Err(e) => HttpResponse::BadRequest().json(ApiError::new(400, e)),
    }
}

pub async fn update_password(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<UpdatePasswordRequest>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let uid: i64 = claims.sub.parse().unwrap_or(0);

    // Ambil user untuk verifikasi password lama
    let user = match state.db.get_user_by_id(uid).await {
        Ok(Some(u)) => u,
        _ => return HttpResponse::NotFound().json(ApiError::new(404, "User tidak ditemukan")),
    };

    let hash = user.password_hash.clone();
    let pw   = body.current_password.clone();
    let ok   = tokio::task::spawn_blocking(move || verify_password(&pw, &hash))
        .await
        .unwrap_or(Ok(false))
        .unwrap_or(false);

    if !ok {
        return HttpResponse::Unauthorized()
            .json(ApiError::new(401, "Password saat ini tidak sesuai"));
    }

    if body.new_password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Password baru minimal 8 karakter"));
    }

    let new_hash = match tokio::task::spawn_blocking({
        let pw = body.new_password.clone();
        move || hash_password(&pw)
    }).await {
        Ok(Ok(h)) => h,
        _ => return HttpResponse::InternalServerError()
            .json(ApiError::new(500, "Gagal memproses password")),
    };

    match state.db.update_user_password(uid, &new_hash).await {
        Ok(_)  => HttpResponse::Ok().json(ApiSuccess::new("Password berhasil diperbarui")),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

// ══════════════════════════════════════════════════════════════════
//  RAG / CHAT HANDLERS
// ══════════════════════════════════════════════════════════════════

pub async fn ask(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<AskRequest>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let uid: i64 = claims.sub.parse().unwrap_or(0);

    if body.query.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Query tidak boleh kosong"));
    }

    // Ambil atau buat session
    let session_id = body.session_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut session = state.db
        .get_session(&session_id, uid)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| ChatSession {
            id:         session_id.clone(),
            user_id:    uid,
            title:      body.query.chars().take(50).collect(),
            messages:   vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        });

    // Konversi history ke format Message untuk RAG
    let history: Vec<crate::rag::Message> = session.messages.iter().map(|m| {
        crate::rag::Message { role: m.role.clone(), content: m.content.clone() }
    }).collect();

    let search_mode = body.search_mode.as_deref().unwrap_or("hybrid");
    let top_k       = body.top_k.unwrap_or(5);
    let cat_filter  = body.category_filter.as_deref();

    let start = std::time::Instant::now();

    let (answer, chunks) = match state.rag.answer(
        &body.query,
        &history,
        cat_filter,
        search_mode,
        top_k,
    ).await {
        Ok(r)  => r,
        Err(e) => return HttpResponse::InternalServerError()
            .json(ApiError::new(500, format!("Gagal mendapat jawaban: {}", e))),
    };

    let elapsed_ms = start.elapsed().as_millis() as u64;

    // Simpan ke session
    let now = Utc::now().to_rfc3339();
    session.messages.push(ChatMessage {
        role:       "user".to_string(),
        content:    body.query.clone(),
        created_at: now.clone(),
        sources:    vec![],
    });
    session.messages.push(ChatMessage {
        role:       "assistant".to_string(),
        content:    answer.clone(),
        created_at: now.clone(),
        sources:    chunks.iter().map(|c| ChatSource {
            title:      c.title.clone(),
            snippet:    c.snippet.clone(),
            source_url: c.source_url.clone(),
            category:   c.category.clone(),
            score:      c.score,
        }).collect(),
    });
    session.updated_at = now;

    let _ = state.db.save_session(&session).await;

    // Log query
    let top_score = chunks.first().map(|c| c.score as f32).unwrap_or(0.0);
    let _ = state.db.log_query(
        &body.query,
        "id",
        cat_filter.unwrap_or(""),
        chunks.len() as i32,
        top_score,
        elapsed_ms as i64,
        Some(&session_id),
        Some(uid),
    ).await;

    // Build sources
    let sources: Vec<SourceRef> = chunks.iter().map(|c| SourceRef {
        title:      c.title.clone(),
        snippet:    c.snippet.clone(),
        source_url: c.source_url.clone(),
        category:   c.category.clone(),
        score:      c.score,
    }).collect();

    HttpResponse::Ok().json(ApiSuccess::new(AskResponse {
        answer,
        session_id,
        sources,
        search_time_ms: elapsed_ms,
    }))
}

// ── Session management ──

pub async fn list_sessions(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.get_user_sessions(uid).await {
        Ok(sessions) => HttpResponse::Ok().json(ApiSuccess::new(sessions)),
        Err(e)       => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn get_session_handler(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.get_session(&path.into_inner(), uid).await {
        Ok(Some(s)) => HttpResponse::Ok().json(ApiSuccess::new(s)),
        Ok(None)    => HttpResponse::NotFound().json(ApiError::new(404, "Session tidak ditemukan")),
        Err(e)      => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn delete_session_handler(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.delete_session(&path.into_inner(), uid).await {
        Ok(true)  => HttpResponse::Ok().json(ApiSuccess::new("Session dihapus")),
        Ok(false) => HttpResponse::NotFound().json(ApiError::new(404, "Session tidak ditemukan")),
        Err(e)    => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn rename_session_handler(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<String>,
    body:  web::Json<serde_json::Value>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("");
    match state.db.rename_session(&path.into_inner(), uid, title).await {
        Ok(true)  => HttpResponse::Ok().json(ApiSuccess::new("Session diubah")),
        Ok(false) => HttpResponse::NotFound().json(ApiError::new(404, "Session tidak ditemukan")),
        Err(e)    => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

// ══════════════════════════════════════════════════════════════════
//  API KEY HANDLERS
// ══════════════════════════════════════════════════════════════════

pub async fn create_api_key_handler(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<CreateApiKeyRequest>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);

    let (full_key, prefix, key_hash) = match generate_api_key() {
        Ok(k)  => k,
        Err(_) => return HttpResponse::InternalServerError()
            .json(ApiError::new(500, "Gagal generate API key")),
    };

    let default_perms = vec!["search", "read"];
    let perms: Vec<&str> = body.permissions
        .as_ref()
        .map(|p| p.iter().map(|s| s.as_str()).collect())
        .unwrap_or(default_perms);

    let rate_limit = body.rate_limit.unwrap_or(30);

    match state.db.create_api_key(
        uid,
        &prefix,
        &key_hash,
        &body.name,
        &perms,
        rate_limit,
        body.expires_at.as_deref(),
    ).await {
        Ok(id) => HttpResponse::Created().json(ApiSuccess::new(CreateApiKeyResponse {
            id,
            key: full_key,   // ditampilkan SEKALI saja
            key_prefix: prefix,
            name: body.name.clone(),
        })),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn list_api_keys(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.get_api_keys(uid).await {
        Ok(keys) => HttpResponse::Ok().json(ApiSuccess::new(keys)),
        Err(e)   => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn revoke_api_key_handler(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<i64>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    let uid: i64 = claims.sub.parse().unwrap_or(0);
    match state.db.revoke_api_key(path.into_inner(), uid).await {
        Ok(true)  => HttpResponse::Ok().json(ApiSuccess::new("API key dinonaktifkan")),
        Ok(false) => HttpResponse::NotFound().json(ApiError::new(404, "API key tidak ditemukan")),
        Err(e)    => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

// ══════════════════════════════════════════════════════════════════
//  ADMIN / STATS HANDLERS
// ══════════════════════════════════════════════════════════════════

pub async fn query_stats(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin yang bisa akses"));
    }
    match state.db.get_query_log_stats().await {
        Ok(stats) => HttpResponse::Ok().json(ApiSuccess::new(stats)),
        Err(e)    => HttpResponse::InternalServerError()
            .json(ApiError::new(500, format!("{}", e))),
    }
}

pub async fn list_categories_handler(
    state: web::Data<AppState>,
) -> HttpResponse {
    match state.db.list_categories().await {
        Ok(cats) => HttpResponse::Ok().json(ApiSuccess::new(cats)),
        Err(e)   => HttpResponse::InternalServerError()
            .json(ApiError::new(500, format!("{}", e))),
    }
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "pmpsti-rag-api"
    }))
}