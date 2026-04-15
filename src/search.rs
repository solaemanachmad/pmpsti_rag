use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use log::info;
use std::sync::{Arc, Mutex};

use crate::db::{Database, SearchResult};

pub struct SearchEngine {
    pub db: Arc<Database>,
    model:  Mutex<TextEmbedding>,
}

impl SearchEngine {
    pub fn new(db: Arc<Database>) -> Result<Self, anyhow::Error> {
        info!("Loading multilingual-e5-base model (fastembed)...");
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::MultilingualE5Base)
        )?;
        info!("AI model loaded successfully.");
        Ok(Self { db, model: Mutex::new(model) })
    }

    fn embed_query(&self, query: &str) -> Result<Vec<f32>, anyhow::Error> {
        let prefixed = format!("query: {}", query);
        let mut lock = self.model.lock().unwrap();
        let vecs = lock.embed(vec![prefixed], None)?;
        Ok(vecs[0].clone())
    }

    // ── Hybrid (default) ──
    pub async fn search(
        &self,
        query:           &str,
        limit:           i64,
        category_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>, anyhow::Error> {
        let embedding = self.embed_query(query)?;
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
        let embedding = self.embed_query(query)?;
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

    // ── Ganti snippet default dengan snippet berbasis query terms ──
    fn enrich_snippets(&self, results: &mut Vec<SearchResult>, query: &str) {
        let terms: Vec<&str> = query.split_whitespace().collect();
        for r in results.iter_mut() {
            r.snippet = Database::extract_snippet(&r.content, &terms, 400);
        }
    }

    // ── Format context untuk dikirim ke LLM ──
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