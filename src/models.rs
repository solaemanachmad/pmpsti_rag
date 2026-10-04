use serde::{Deserialize, Serialize};

// ══════════════════════════════════════════════════════════════════
//  JWT CLAIMS
// ══════════════════════════════════════════════════════════════════

/// Payload yang di-encode ke dalam JWT token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub:   String,   // user id (sebagai string)
    pub email: String,
    pub role:  String,
    pub exp:   usize,    // unix timestamp expiry
    pub iat:   usize,    // unix timestamp issued-at
}

// ══════════════════════════════════════════════════════════════════
//  AUTH REQUEST / RESPONSE
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email:        String,
    pub password:     String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token:        String,
    pub token_type:   String,   // "Bearer"
    pub expires_in:   u64,      // seconds
    pub user:         UserPublic,
}

/// Versi publik User — tanpa password_hash
#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id:           i64,
    pub email:        String,
    pub display_name: String,
    pub role:         String,
    pub created_at:   String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub email:        Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password:     String,
}

// ══════════════════════════════════════════════════════════════════
//  CHAT REQUEST / RESPONSE
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AskRequest {
    pub query:           String,
    pub session_id:      Option<String>,
    pub category_filter: Option<String>,
    pub search_mode:     Option<String>,  // "hybrid" | "semantic" | "keyword"
    pub top_k:           Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub answer:     String,
    pub session_id: String,
    pub sources:    Vec<SourceRef>,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SourceRef {
    pub title:      String,
    pub snippet:    String,
    pub source_url: String,
    pub category:   String,
    pub subcategory: String,
    pub score:      f64,
}

// ══════════════════════════════════════════════════════════════════
//  API KEY REQUEST / RESPONSE
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name:        String,
    pub permissions: Option<Vec<String>>,
    pub rate_limit:  Option<i32>,
    pub expires_at:  Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateApiKeyResponse {
    pub id:         i64,
    pub key:        String,   // full key — HANYA ditampilkan sekali saat create
    pub key_prefix: String,
    pub name:       String,
}

// ══════════════════════════════════════════════════════════════════
//  GENERIC RESPONSE WRAPPERS
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize)]
pub struct ApiSuccess<T: Serialize> {
    pub success: bool,
    pub data:    T,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error:   String,
    pub code:    u16,
}

impl<T: Serialize> ApiSuccess<T> {
    pub fn new(data: T) -> Self {
        Self { success: true, data }
    }
}

impl ApiError {
    pub fn new(code: u16, error: impl Into<String>) -> Self {
        Self { success: false, error: error.into(), code }
    }
}
// ══════════════════════════════════════════════════════════════════
//  DOCUMENT INGEST REQUESTS
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct IngestUrlRequest {
    pub url:         String,
    pub title:       Option<String>,
    pub category:    Option<String>,
    pub subcategory: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub document_id: String,
    pub chunks:      usize,
    pub title:       String,
}
