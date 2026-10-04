use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use crate::auth::{extract_bearer, generate_api_key, hash_password, sign_jwt, verify_jwt, verify_password};
use crate::db::{ChatMessage, ChatSession, ChatSource, Database};
use crate::email::send_verification_email;
use crate::models::*;
use crate::rag::RagEngine;

// ── Rate limiter state: IP -> (fail_count, window_start) ──
#[derive(Clone)]
pub struct LoginAttempt {
    count:      u32,
    window_start: Instant,
}

const MAX_LOGIN_FAILS:   u32 = 5;
const LOGIN_WINDOW_SECS: u64 = 15 * 60; // 15 menit

// ── App state yang di-share ke semua handler ──
pub struct AppState {
    pub db:           Arc<Database>,
    pub rag:          Arc<RagEngine>,
    pub jwt_secret:   String,
    pub resend_key:   String,
    pub app_base_url: String,
    pub login_attempts: Arc<DashMap<String, LoginAttempt>>,
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
    // Validasi dasar
    if body.email.is_empty() || body.password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Email tidak valid atau password kurang dari 8 karakter"));
    }

    // Hanya izinkan email @mail.ugm.ac.id
    if !body.email.to_lowercase().ends_with("@mail.ugm.ac.id") {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Registrasi hanya untuk email @mail.ugm.ac.id"));
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
    let user = match state.db.create_user(&body.email, &hash, &display).await {
        Ok(u)  => u,
        Err(e) => return HttpResponse::Conflict().json(ApiError::new(409, e)),
    };

    // Buat token verifikasi dan kirim email
    let token = Uuid::new_v4().to_string();
    if let Err(e) = state.db.create_verification_token(user.id, &token).await {
        log::error!("Gagal menyimpan verification token: {e}");
        return HttpResponse::InternalServerError()
            .json(ApiError::new(500, "Gagal membuat token verifikasi"));
    }

    let verify_url = format!("{}/verify-email?token={}", state.app_base_url, token);
    if let Err(e) = send_verification_email(
        &state.resend_key,
        &user.email,
        &verify_url,
        &state.app_base_url,
    )
    .await
    {
        log::error!("Gagal mengirim email verifikasi: {e}");
        return HttpResponse::InternalServerError()
            .json(ApiError::new(500, "Gagal mengirim email verifikasi"));
    }

    HttpResponse::Created().json(ApiSuccess::new(serde_json::json!({
        "message": "Registrasi berhasil. Silakan cek email @mail.ugm.ac.id Anda untuk verifikasi akun."
    })))
}

// ── Verifikasi email via token ──
pub async fn verify_email(
    state: web::Data<AppState>,
    path:  web::Path<String>,
) -> HttpResponse {
    let token = path.into_inner();
    match state.db.verify_email_token(&token).await {
        Ok(Some(_)) => HttpResponse::Ok().json(ApiSuccess::new(serde_json::json!({
            "message": "Email berhasil diverifikasi. Silakan login."
        }))),
        Ok(None) => HttpResponse::BadRequest().json(ApiError::new(
            400,
            "Token verifikasi tidak valid atau sudah kedaluwarsa",
        )),
        Err(e) => {
            log::error!("verify_email_token error: {e}");
            HttpResponse::InternalServerError()
                .json(ApiError::new(500, "Gagal memverifikasi email"))
        }
    }
}

pub async fn login(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<LoginRequest>,
) -> HttpResponse {
    // ── Rate limiting: max 5 gagal per 15 menit per IP ──
    let ip = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    {
        let now = Instant::now();
        let mut entry = state.login_attempts.entry(ip.clone()).or_insert_with(|| LoginAttempt {
            count: 0,
            window_start: now,
        });
        if entry.window_start.elapsed().as_secs() >= LOGIN_WINDOW_SECS {
            entry.count = 0;
            entry.window_start = now;
        }
        if entry.count >= MAX_LOGIN_FAILS {
            let remaining = LOGIN_WINDOW_SECS
                .saturating_sub(entry.window_start.elapsed().as_secs());
            return HttpResponse::TooManyRequests().json(ApiError::new(
                429,
                &format!(
                    "Terlalu banyak percobaan login. Coba lagi dalam {} menit.",
                    (remaining + 59) / 60
                ),
            ));
        }
    }

    let user = match state.db.get_user_by_email(&body.email).await {
        Ok(Some(u)) => u,
        _ => {
            // Increment fail counter agar tidak bisa brute-force enumeration
            if let Some(mut entry) = state.login_attempts.get_mut(&ip) {
                entry.count += 1;
            }
            return HttpResponse::Unauthorized()
                .json(ApiError::new(401, "Email atau password salah"));
        }
    };

    // Cek apakah email sudah diverifikasi
    if !user.email_verified {
        return HttpResponse::Forbidden().json(ApiError::new(
            403,
            "Akun belum diverifikasi. Silakan cek email Anda untuk link verifikasi.",
        ));
    }

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
        if let Some(mut entry) = state.login_attempts.get_mut(&ip) {
            entry.count += 1;
        }
        return HttpResponse::Unauthorized()
            .json(ApiError::new(401, "Email atau password salah"));
    }

    // Login berhasil — reset counter
    state.login_attempts.remove(&ip);

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


// ══════════════════════════════════════════════════════════════════
//  FILTER SOURCES — dynamic relevance filtering
// ══════════════════════════════════════════════════════════════════

/// Filter chunks hasil retrieval menjadi referensi yang benar-benar relevan.
///
/// Strategi (berurutan):
/// 1. Score threshold  — buang chunk dengan score < min_score
/// 2. Score gap        — potong jika ada drop > gap_ratio antara chunk berurutan
/// 3. Dedup per source — satu source_url hanya muncul sekali (chunk terbaik)
/// 4. Max cap          — paling banyak max_sources referensi
fn filter_sources(
    chunks: &[crate::db::SearchResult],
    min_score:   f64,
    gap_ratio:   f64,
    max_sources: usize,
) -> Vec<&crate::db::SearchResult> {
    if chunks.is_empty() {
        return vec![];
    }

    // 1. Score threshold
    let above: Vec<&crate::db::SearchResult> = chunks.iter()
        .filter(|c| c.score >= min_score)
        .collect();

    if above.is_empty() {
        // Fallback: kembalikan chunk terbaik saja (daripada kosong)
        return vec![&chunks[0]];
    }

    // 2. Score gap — cari titik drop pertama yang signifikan
    let mut cutoff = above.len();
    for i in 1..above.len() {
        let prev = above[i - 1].score;
        let curr = above[i].score;
        if prev > 0.0 && (prev - curr) / prev > gap_ratio {
            cutoff = i;
            break;
        }
    }
    let filtered: Vec<&crate::db::SearchResult> = above[..cutoff].to_vec();

    // 3. Dedup per source_url (ambil score tertinggi per URL)
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();
    let deduped: Vec<&crate::db::SearchResult> = filtered.into_iter()
        .filter(|c| {
            let key = if c.source_url.is_empty() {
                c.title.clone()
            } else {
                c.source_url.clone()
            };
            seen_urls.insert(key)
        })
        .collect();

    // 4. Max cap
    deduped.into_iter().take(max_sources).collect()
}

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
    let top_k       = body.top_k.unwrap_or(10); // fetch lebih banyak, filter di filter_sources  // fetch lebih banyak, lalu filter
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

    // Filter chunks → hanya referensi yang benar-benar relevan
    // min_score=0.40 (sedikit lebih lebar), gap_ratio=0.25, max=6
    let relevant = filter_sources(&chunks, 0.40, 0.25, 6);

    // Simpan ke session (gunakan relevant chunks saja)
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
        sources:    relevant.iter().map(|c| ChatSource {
            title:      clean_source_title(&c.title, &c.subcategory),
            snippet:    c.snippet.clone(),
            source_url: c.source_url.clone(),
            category:   c.category.clone(),
            subcategory: c.subcategory.clone(),
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
        relevant.len() as i32,
        top_score,
        elapsed_ms as i64,
        Some(&session_id),
        Some(uid),
    ).await;

    // Build sources
    let sources: Vec<SourceRef> = relevant.iter().map(|c| SourceRef {
        title:      clean_source_title(&c.title, &c.subcategory),
        snippet:    c.snippet.clone(),
        source_url: c.source_url.clone(),
        category:   c.category.clone(),
        subcategory: c.subcategory.clone(),
        score:      c.score,
    }).collect();

    HttpResponse::Ok().json(ApiSuccess::new(AskResponse {
        answer,
        session_id,
        sources,
        search_time_ms: elapsed_ms,
    }))
}


// ══════════════════════════════════════════════════════════════════
//  PUBLIC ASK — tanpa auth, maks GUEST_QUOTA pertanyaan per token
// ══════════════════════════════════════════════════════════════════

const GUEST_QUOTA: i32 = 5;

pub async fn ask_public(
    req:   HttpRequest,
    state: web::Data<AppState>,
    body:  web::Json<AskRequest>,
) -> HttpResponse {
    // Guest token dari header X-Guest-Token
    let guest_token = req
        .headers()
        .get("X-Guest-Token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if guest_token.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "X-Guest-Token header diperlukan"));
    }

    // Cek quota
    let count = state.db.guest_ask_count(&guest_token).await;
    if count >= GUEST_QUOTA {
        return HttpResponse::TooManyRequests()
            .json(ApiError::new(429, format!(
                "Batas {} pertanyaan untuk tamu telah habis. Silakan daftar atau masuk.",
                GUEST_QUOTA
            )));
    }

    if body.query.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiError::new(400, "Query tidak boleh kosong"));
    }

    let search_mode = body.search_mode.as_deref().unwrap_or("hybrid");
    let top_k       = body.top_k.unwrap_or(5);

    let start = std::time::Instant::now();

    let (answer, chunks) = match state.rag.answer(
        &body.query,
        &[],            // no history for guests
        None,
        search_mode,
        top_k,
    ).await {
        Ok(r)  => r,
        Err(e) => return HttpResponse::InternalServerError()
            .json(ApiError::new(500, format!("Gagal mendapat jawaban: {}", e))),
    };

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let relevant   = filter_sources(&chunks, 0.45, 0.25, 5);

    // Increment setelah berhasil menjawab
    let new_count = state.db.guest_increment(&guest_token).await;

    let sources: Vec<SourceRef> = relevant.iter().map(|c| SourceRef {
        title:      clean_source_title(&c.title, &c.subcategory),
        snippet:    c.snippet.clone(),
        source_url: c.source_url.clone(),
        category:   c.category.clone(),
        subcategory: c.subcategory.clone(),
        score:      c.score,
    }).collect();

    HttpResponse::Ok().json(ApiSuccess::new(serde_json::json!({
        "answer":          answer,
        "sources":         sources,
        "search_time_ms":  elapsed_ms,
        "questions_used":  new_count,
        "questions_left":  (GUEST_QUOTA - new_count).max(0),
    })))
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

// ══════════════════════════════════════════════════════════════════
//  ADMIN — USER MANAGEMENT
// ══════════════════════════════════════════════════════════════════

pub async fn admin_list_users(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    match state.db.admin_list_users().await {
        Ok(users) => HttpResponse::Ok().json(ApiSuccess::new(users)),
        Err(e)    => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn admin_set_user_role(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<i64>,
    body:  web::Json<serde_json::Value>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    let role = body.get("role").and_then(|v| v.as_str()).unwrap_or("user");
    match state.db.admin_set_role(path.into_inner(), role).await {
        Ok(_)  => HttpResponse::Ok().json(ApiSuccess::new("Role diperbarui")),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn admin_toggle_user(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<i64>,
    body:  web::Json<serde_json::Value>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    let active = body.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);
    match state.db.admin_set_active(path.into_inner(), active).await {
        Ok(_)  => HttpResponse::Ok().json(ApiSuccess::new("Status user diperbarui")),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

// ══════════════════════════════════════════════════════════════════
//  ADMIN — CHAT SESSIONS
// ══════════════════════════════════════════════════════════════════

pub async fn admin_list_sessions(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    match state.db.admin_list_sessions().await {
        Ok(sessions) => HttpResponse::Ok().json(ApiSuccess::new(sessions)),
        Err(e)       => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn admin_delete_session(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    match state.db.admin_delete_session(&path.into_inner()).await {
        Ok(true)  => HttpResponse::Ok().json(ApiSuccess::new("Session dihapus")),
        Ok(false) => HttpResponse::NotFound().json(ApiError::new(404, "Session tidak ditemukan")),
        Err(e)    => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

// ══════════════════════════════════════════════════════════════════
//  ADMIN — DOCUMENTS
// ══════════════════════════════════════════════════════════════════

pub async fn admin_list_documents(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    match state.db.admin_list_documents().await {
        Ok(docs) => HttpResponse::Ok().json(ApiSuccess::new(docs)),
        Err(e)   => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

pub async fn admin_delete_document(
    req:   HttpRequest,
    state: web::Data<AppState>,
    path:  web::Path<String>,
) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    match state.db.admin_delete_document(&path.into_inner()).await {
        Ok(n)  => HttpResponse::Ok().json(ApiSuccess::new(
            serde_json::json!({ "deleted_chunks": n })
        )),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, e)),
    }
}

/// Bersihkan judul sumber referensi untuk tampilan di frontend.
/// Prioritaskan subcategory (misal "1.2 Latar Belakang") daripada path title
/// (misal "chapter1/introduction"). Jika subcategory kosong, ambil bagian terakhir
/// dari title (setelah `/`), atau title itu sendiri.
fn clean_source_title(title: &str, subcategory: &str) -> String {
    // Subcategory biasanya format "1.2Latar Belakang" atau "1.2 Latar Belakang"
    // Tambah spasi antara angka dan huruf jika belum ada
    if !subcategory.trim().is_empty() {
        let sub = subcategory.trim();
        // Insert spasi setelah digit+titik pattern: "1.2Latar" -> "1.2 Latar"
        let re_result = {
            let mut out = String::with_capacity(sub.len() + 4);
            let chars: Vec<char> = sub.chars().collect();
            for (i, &c) in chars.iter().enumerate() {
                out.push(c);
                if i + 1 < chars.len() {
                    let next = chars[i + 1];
                    // After digit or dot, if next is uppercase/letter but no space
                    if (c.is_ascii_digit() || c == '.') && next.is_alphabetic() && next != ' ' {
                        // Check we haven't already got a space
                        out.push(' ');
                    }
                }
            }
            out
        };
        return re_result;
    }
    // Fallback: ambil bagian terakhir setelah '/'
    if let Some(last) = title.rsplit('/').next() {
        let last = last.trim();
        if !last.is_empty() {
            return last.to_string();
        }
    }
    title.to_string()
}


// ── Admin: query logs ──────────────────────────────────────────────
pub async fn admin_query_logs(req: HttpRequest, state: web::Data<AppState>) -> HttpResponse {
    let claims = match require_auth(&req, &state.jwt_secret) { Ok(c) => c, Err(r) => return r };
    if claims.role != "admin" {
        return HttpResponse::Forbidden().json(ApiError::new(403, "Hanya admin"));
    }
    let query = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));
    let limit:  i64 = query.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50).min(200);
    let offset: i64 = query.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0).max(0);

    match state.db.admin_list_query_logs(limit, offset).await {
        Ok((logs, total)) => HttpResponse::Ok().json(ApiSuccess::new(serde_json::json!({
            "logs": logs,
            "total": total,
            "limit": limit,
            "offset": offset,
        }))),
        Err(e) => HttpResponse::InternalServerError().json(ApiError::new(500, format!("{e}"))),
    }
}
