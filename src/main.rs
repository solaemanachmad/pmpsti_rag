mod auth;
mod db;
mod handlers;
mod models;
mod rag;
mod search;

use actix_web::{middleware, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use db::Database;
use handlers::AppState;
use rag::{LlmConfig, RagEngine};
use search::SearchEngine;

// ══════════════════════════════════════════════════════════════════
//  CONFIG  — dibaca dari environment variables
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct Config {
    database_url: String,
    jwt_secret:   String,
    host:         String,
    port:         u16,
    llm_backend:  String,   // "ollama" | "openai" | url custom
    llm_model:    String,
    llm_api_key:  String,
    llm_api_url:  String,
}

impl Config {
    fn from_env() -> Self {
        // Muat .env jika ada (opsional, tidak panik jika tidak ada)
        let _ = dotenvy::dotenv();

        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://admin:admin@localhost:5432/pmpsti_rag".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "ganti-ini-dengan-secret-yang-aman-minimal-32-karakter".to_string()),
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            llm_backend: std::env::var("LLM_BACKEND")
                .unwrap_or_else(|_| "ollama".to_string()),
            llm_model: std::env::var("LLM_MODEL")
                .unwrap_or_else(|_| "llama3.2".to_string()),
            llm_api_key: std::env::var("LLM_API_KEY")
                .unwrap_or_default(),
            llm_api_url: std::env::var("LLM_API_URL")
                .unwrap_or_default(),
        }
    }
}

// ══════════════════════════════════════════════════════════════════
//  MAIN
// ══════════════════════════════════════════════════════════════════

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logger
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let cfg = Config::from_env();

    log::info!("═══════════════════════════════════════");
    log::info!("  PMPSTI RAG API");
    log::info!("  Database : {}", cfg.database_url);
    log::info!("  LLM      : {} ({})", cfg.llm_model, cfg.llm_backend);
    log::info!("  Listen   : {}:{}", cfg.host, cfg.port);
    log::info!("═══════════════════════════════════════");

    // ── Database ──
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await
        .expect("Gagal konek ke PostgreSQL");

    let db = Arc::new(Database::new(pool));

    db.setup()
        .await
        .expect("Gagal setup database schema");

    // ── Search engine ──
    let search = SearchEngine::new(db.clone())
        .expect("Gagal load embedding model");

    // ── LLM config ──
    let llm_config = match cfg.llm_backend.as_str() {
        "ollama" => LlmConfig::ollama(&cfg.llm_model),
        _ => {
            let url = if cfg.llm_api_url.is_empty() {
                "https://api.openai.com/v1/chat/completions".to_string()
            } else {
                cfg.llm_api_url.clone()
            };
            LlmConfig::openai_compatible(&url, &cfg.llm_api_key, &cfg.llm_model)
        }
    };

    // ── RAG engine ──
    let rag = Arc::new(RagEngine::new(search, llm_config));

    // ── App state ──
    let jwt_secret = cfg.jwt_secret.clone();
    let state = web::Data::new(AppState {
        db:         db.clone(),
        rag:        rag.clone(),
        jwt_secret: jwt_secret.clone(),
    });

    let addr = format!("{}:{}", cfg.host, cfg.port);
    log::info!("Server siap di http://{}", addr);

    // ── HTTP Server ──
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| {
                        let msg = format!("JSON parse error: {}", err);
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest()
                                .json(models::ApiError::new(400, msg)),
                        )
                        .into()
                    })
            )
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            // ── Routes ──
            .route("/health", web::get().to(handlers::health))

            // Auth
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(handlers::register))
                    .route("/login",    web::post().to(handlers::login))
                    .route("/me",       web::get().to(handlers::get_me))
                    .route("/me",       web::patch().to(handlers::update_profile))
                    .route("/me/password", web::patch().to(handlers::update_password))
            )

            // RAG / Chat
            .service(
                web::scope("/api")
                    .route("/ask",        web::post().to(handlers::ask))
                    .route("/categories", web::get().to(handlers::list_categories_handler))
                    // Sessions
                    .route("/sessions",           web::get().to(handlers::list_sessions))
                    .route("/sessions/{id}",       web::get().to(handlers::get_session_handler))
                    .route("/sessions/{id}",       web::delete().to(handlers::delete_session_handler))
                    .route("/sessions/{id}/title", web::patch().to(handlers::rename_session_handler))
                    // API Keys
                    .route("/keys",       web::get().to(handlers::list_api_keys))
                    .route("/keys",       web::post().to(handlers::create_api_key_handler))
                    .route("/keys/{id}",  web::delete().to(handlers::revoke_api_key_handler))
                    // Admin
                    .route("/admin/stats", web::get().to(handlers::query_stats))
            )
    })
    .bind(&addr)?
    .run()
    .await
}

// ── Shortcut HttpResponse untuk handler closure ──
use actix_web::HttpResponse;