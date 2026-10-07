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
#[serde(rename_all = "camelCase")]
struct EmbedRequest {
    content: EmbedContent,
    output_dimensionality: u32,
}

#[derive(Serialize)]
struct EmbedContent {
    parts: Vec<EmbedPart>,
}

#[derive(Serialize)]
struct EmbedPart {
    text: String,
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
        info!("SearchEngine siap (Gemini gemini-embedding-001, 768-dim).");
        Ok(Self {
            db,
            api_key,
            http_client: Client::new(),
        })
    }

    async fn embed_query(&self, query: &str) -> Result<Vec<f32>, anyhow::Error> {
        let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-001:embedContent";
        let body = EmbedRequest {
            content: EmbedContent {
                parts: vec![EmbedPart { text: query.to_string() }],
            },
            output_dimensionality: 768,
        };
        let resp = self.http_client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .header("Content-Type", "application/json")
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

    // ── Reciprocal Rank Fusion ──
    pub async fn search_rrf(
        &self,
        query:           &str,
        limit:           i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, anyhow::Error> {
        let embedding = self.embed_query(query).await?;
        let mut results = self.db.search_rrf(query, embedding, limit, category_filter).await?;
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
            .map(|(i, r)| Self::format_one(i, r))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    /// Sama seperti format_context, tapi menerima slice of references
    /// (dipakai oleh answer_with_chunks setelah filter_sources).
    pub fn format_context_refs(results: &[&SearchResult]) -> String {
        results
            .iter()
            .enumerate()
            .map(|(i, r)| Self::format_one(i, r))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    fn format_one(i: usize, r: &SearchResult) -> String {
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
    }
}
