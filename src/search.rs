use log::info;
use std::{env, sync::Arc};
use serde::{Deserialize, Serialize};
use reqwest::Client;

use crate::db::{Database, SearchResult};

pub struct SearchEngine {
    pub db:      Arc<Database>,
    api_key:     String,
    http_client: Client,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model:   &'a str,
    content: EmbedContent<'a>,
}

#[derive(Serialize)]
struct EmbedContent<'a> {
    parts: Vec<EmbedPart<'a>>,
}

#[derive(Serialize)]
struct EmbedPart<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: EmbedValues,
}

#[derive(Deserialize)]
struct EmbedValues {
    values: Vec<f32>,
}

impl SearchEngine {
    pub fn new(db: Arc<Database>) -> Result<Self, anyhow::Error> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| anyhow::anyhow!("GEMINI_API_KEY tidak ditemukan"))?;
        info!("SearchEngine siap (Gemini text-embedding-004).");
        Ok(Self {
            db,
            api_key,
            http_client: Client::new(),
        })
    }

    async fn embed_query(&self, query: &str) -> Result<Vec<f32>, anyhow::Error> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key={}",
            self.api_key
        );
        let body = EmbedRequest {
            model: "models/text-embedding-004",
            content: EmbedContent {
                parts: vec![EmbedPart { text: query }],
            },
        };
        let resp = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<EmbedResponse>()
            .await?;
        Ok(resp.embedding.values)
    }

    // ── Hybrid (default) ──
    pub async fn search(
        &self,
        query:           &str,
        limit:           i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, anyhow::Error> {
        let embedding = self.embed_query(query).await?;
        let mut results = self.db.search_hybrid(query, embedding, limit, category_filter).await?;
        self.enrich_snippets(&mut results, query);
        Ok(results)
    }

    // ── Pure vector ──
    pub async fn search_semantic(
        &self,
        query:           &str,
        limit:           i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, anyhow::Error> {
        let embedding = self.embed_query(query).await?;
        let mut results = self.db.search_vector(embedding, limit, category_filter).await?;
        self.enrich_snippets(&mut results, query);
        Ok(results)
    }

    // ── Pure fulltext ──
    pub async fn search_keyword(
        &self,
        query:           &str,
        limit:           i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, anyhow::Error> {
        let mut results = self.db.search_fulltext(query, limit, category_filter).await?;
        self.enrich_snippets(&mut results, query);
        Ok(results)
    }

    fn enrich_snippets(&self, results: &mut Vec<SearchResult>, query: &str) {
        let terms: Vec<&str> = query.split_whitespace().collect();
        for r in results.iter_mut() {
            r.snippet = Database::extract_snippet(&r.content, &terms, 400);
        }
    }

    pub fn format_context(results: &[SearchResult]) -> String {
        results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let location = match r.page_number {
                    Some(p) => format!("hal. {}", p),
                    None    => "—".to_string(),
                };
                format!(
                    "[{}] {} ({})\nKategori: {} / {}\nSumber: {}\n\n{}",
                    i + 1,
                    r.title,
                    location,
                    r.category,
                    r.subcategory,
                    r.source_url,
                    r.content,
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }
}
