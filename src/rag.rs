use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::db::SearchResult;
use crate::search::SearchEngine;

// ══════════════════════════════════════════════════════════════════
//  CONFIG
// ══════════════════════════════════════════════════════════════════

/// Konfigurasi LLM backend — bisa Ollama lokal atau OpenAI-compatible.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub api_url:   String,   // e.g. "http://localhost:11434/api/chat" (Ollama)
                             //   atau "https://api.openai.com/v1/chat/completions"
    pub api_key:   String,   // kosong jika Ollama lokal
    pub model:     String,   // e.g. "llama3.2", "gpt-4o-mini", "claude-sonnet-4-6"
    pub max_tokens: u32,
}

impl LlmConfig {
    /// Ollama lokal (default)
    pub fn ollama(model: &str) -> Self {
        Self {
            api_url:    "http://localhost:11434/api/chat".to_string(),
            api_key:    String::new(),
            model:      model.to_string(),
            max_tokens: 2048,
        }
    }

    /// OpenAI-compatible (OpenAI, Groq, Together, dsb.)
    pub fn openai_compatible(api_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            api_url:    api_url.to_string(),
            api_key:    api_key.to_string(),
            model:      model.to_string(),
            max_tokens: 2048,
        }
    }
}

// ══════════════════════════════════════════════════════════════════
//  OPENAI-COMPATIBLE TYPES
// ══════════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct ChatRequest {
    model:       String,
    messages:    Vec<Message>,
    max_tokens:  u32,
    temperature: f32,
    stream:      bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role:    String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

// Ollama format (berbeda dari OpenAI)
#[derive(Serialize)]
struct OllamaChatRequest {
    model:    String,
    messages: Vec<Message>,
    stream:   bool,
    options:  OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: u32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Message,
}

// ══════════════════════════════════════════════════════════════════
//  RAG ENGINE
// ══════════════════════════════════════════════════════════════════

pub struct RagEngine {
    pub search: SearchEngine,
    llm:        LlmConfig,
    http:       Client,
    is_ollama:  bool,
}

impl RagEngine {
    pub fn new(search: SearchEngine, llm: LlmConfig) -> Self {
        let is_ollama = llm.api_url.contains("11434") || llm.api_url.contains("ollama");
        Self {
            search,
            llm,
            http: Client::new(),
            is_ollama,
        }
    }

    // ════════════════════════════════════════════════════════════
    //  MAIN: Answer query dengan RAG pipeline
    //  Returns (answer, sources)
    // ════════════════════════════════════════════════════════════
    pub async fn answer(
        &self,
        query:           &str,
        history:         &[Message],        // riwayat chat (bisa kosong)
        category_filter: Option<&str>,
        search_mode:     &str,              // "hybrid" | "semantic" | "keyword"
        top_k:           i64,
    ) -> Result<(String, Vec<SearchResult>)> {
        // 1. Retrieve chunks
        let chunks = match search_mode {
            "semantic" => self.search.search_semantic(query, top_k, category_filter).await?,
            "keyword"  => self.search.search_keyword(query, top_k, category_filter).await?,
            _          => self.search.search(query, top_k, category_filter).await?,
        };

        if chunks.is_empty() {
            return Ok((
                "Maaf, saya tidak menemukan informasi yang relevan dengan pertanyaan tersebut \
                 dalam dokumen yang tersedia.".to_string(),
                vec![],
            ));
        }

        // 2. Build context
        let context = SearchEngine::format_context(&chunks);

        // 3. Build prompt
        let system_prompt = build_system_prompt();
        let user_message  = build_user_message(query, &context);

        // 4. Susun messages: system + history + user baru
        let mut messages: Vec<Message> = vec![
            Message { role: "system".to_string(), content: system_prompt },
        ];
        // Ambil max 6 pesan terakhir dari history (3 giliran) agar tidak overflow context
        let history_window = if history.len() > 6 { &history[history.len() - 6..] } else { history };
        messages.extend_from_slice(history_window);
        messages.push(Message { role: "user".to_string(), content: user_message });

        // 5. Call LLM
        let answer = if self.is_ollama {
            self.call_ollama(&messages).await?
        } else {
            self.call_openai_compatible(&messages).await?
        };

        Ok((answer, chunks))
    }

    // ════════════════════════════════════════════════════════════
    //  CALL OLLAMA
    // ════════════════════════════════════════════════════════════
    async fn call_ollama(&self, messages: &[Message]) -> Result<String> {
        let req = OllamaChatRequest {
            model:    self.llm.model.clone(),
            messages: messages.to_vec(),
            stream:   false,
            options:  OllamaOptions {
                num_predict: self.llm.max_tokens,
                temperature: 0.3,
            },
        };

        let resp = self.http
            .post(&self.llm.api_url)
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body));
        }

        let data: OllamaChatResponse = resp.json().await?;
        Ok(data.message.content)
    }

    // ════════════════════════════════════════════════════════════
    //  CALL OPENAI-COMPATIBLE
    // ════════════════════════════════════════════════════════════
    async fn call_openai_compatible(&self, messages: &[Message]) -> Result<String> {
        let req = ChatRequest {
            model:       self.llm.model.clone(),
            messages:    messages.to_vec(),
            max_tokens:  self.llm.max_tokens,
            temperature: 0.3,
            stream:      false,
        };

        let mut builder = self.http
            .post(&self.llm.api_url)
            .json(&req);

        if !self.llm.api_key.is_empty() {
            builder = builder.bearer_auth(&self.llm.api_key);
        }

        let resp = builder.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("LLM API error {}: {}", status, body));
        }

        let data: ChatResponse = resp.json().await?;
        data.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("LLM returned empty choices"))
    }
}

// ══════════════════════════════════════════════════════════════════
//  PROMPT BUILDERS
// ══════════════════════════════════════════════════════════════════

fn build_system_prompt() -> String {
    "Kamu adalah asisten Akademik di Universitas untuk menjawab pertanyaan berdasarkan dokumen. \
     \n\nAturan:\n\
     1. Kamu hanya boleh menjawab menggunakan informasi dari teks 'Konteks' yang diberikan.\n\
     2. Jika jawaban dari pertanyaan user tidak ada di dalam Konteks, jawablah dengan: 'Berdasarkan dokumen yang tersedia, informasi tersebut tidak ditemukan.' (Jangan mengarang jawaban di luar konteks).\n\
     3. Selalu sebutkan sumber dengan format [nomor] di akhir kalimat yang kamu kutip."
        .to_string()
}

fn build_user_message(query: &str, context: &str) -> String {
    format!(
        "Konteks dari dokumen:\n\
         ─────────────────────\n\
         {context}\n\
         ─────────────────────\n\
         \n\
         Pertanyaan: {query}"
    )
}