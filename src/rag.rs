use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::db::SearchResult;
use crate::search::SearchEngine;

// ══════════════════════════════════════════════════════════════════
//  CONFIG
// ══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum LlmProvider {
    Ollama,
    OpenAICompatible,
    Gemini,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider:   LlmProvider,
    pub api_url:    String,
    pub api_key:    String,
    pub model:      String,
    pub max_tokens: u32,
}

impl LlmConfig {
    pub fn ollama(model: &str) -> Self {
        Self {
            provider:   LlmProvider::Ollama,
            api_url:    "http://localhost:11434/api/chat".to_string(),
            api_key:    String::new(),
            model:      model.to_string(),
            max_tokens: 2048,
        }
    }

    pub fn openai_compatible(api_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            provider:   LlmProvider::OpenAICompatible,
            api_url:    api_url.to_string(),
            api_key:    api_key.to_string(),
            model:      model.to_string(),
            max_tokens: 2048,
        }
    }

    pub fn gemini(api_key: &str, model: &str) -> Self {
        Self {
            provider:   LlmProvider::Gemini,
            api_url:    String::new(),
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

// ── Ollama ──────────────────────────────────────────────────────

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

// ── Gemini ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct GeminiRequest {
    system_instruction: GeminiSystemInstruction,
    contents:           Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config:  GeminiGenerationConfig,
}

#[derive(Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role:  String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
    temperature:       f32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

// ══════════════════════════════════════════════════════════════════
//  RAG ENGINE
// ══════════════════════════════════════════════════════════════════

pub struct RagEngine {
    pub search: SearchEngine,
    llm:        LlmConfig,
    http:       Client,
}

impl RagEngine {
    pub fn new(search: SearchEngine, llm: LlmConfig) -> Self {
        Self { search, llm, http: Client::new() }
    }

    pub async fn answer(
        &self,
        query:           &str,
        history:         &[Message],
        category_filter: Option<&str>,
        search_mode:     &str,
        top_k:           i64,
    ) -> Result<(String, Vec<SearchResult>)> {
        let initial_chunks = match search_mode {
            "semantic" => self.search.search_semantic(query, top_k, category_filter).await?,
            "keyword"  => self.search.search_keyword(query, top_k, category_filter).await?,
            _          => self.search.search(query, top_k, category_filter).await?,
        };

        if initial_chunks.is_empty() {
            return Ok((
                "Maaf, saya tidak menemukan informasi yang relevan dengan pertanyaan tersebut \
                 dalam dokumen yang tersedia.".to_string(),
                vec![],
            ));
        }

        // Document expansion: jika satu dokumen muncul >= 2x di hasil search,
        // fetch SEMUA chunk dokumen itu (berguna untuk tabel/daftar panjang).
        let mut doc_hit_count: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for c in &initial_chunks {
            if !c.document_id.is_empty() {
                *doc_hit_count.entry(c.document_id.clone()).or_insert(0) += 1;
            }
        }
        let expand_ids: Vec<String> = doc_hit_count
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(id, _)| id)
            .collect();

        let chunks = if expand_ids.is_empty() {
            initial_chunks
        } else {
            // Fetch semua chunk untuk dokumen yang banyak match
            let mut expanded = self.search.db
                .fetch_all_chunks_for_documents(&expand_ids)
                .await
                .unwrap_or_default();
            // Tambahkan chunk dari dokumen lain (tidak di-expand)
            for c in initial_chunks {
                if !expand_ids.contains(&c.document_id) {
                    expanded.push(c);
                }
            }
            expanded
        };

        let context = SearchEngine::format_context(&chunks);
        let system_prompt = build_system_prompt();
        let user_message  = build_user_message(query, &context);

        let mut messages: Vec<Message> = vec![
            Message { role: "system".to_string(), content: system_prompt.clone() },
        ];
        let history_window = if history.len() > 6 { &history[history.len() - 6..] } else { history };
        messages.extend_from_slice(history_window);
        messages.push(Message { role: "user".to_string(), content: user_message });

        let answer = match self.llm.provider {
            LlmProvider::Ollama           => self.call_ollama(&messages).await?,
            LlmProvider::Gemini           => self.call_gemini(&system_prompt, &messages).await?,
            LlmProvider::OpenAICompatible => self.call_openai_compatible(&messages).await?,
        };

        Ok((answer, chunks))
    }

    async fn call_ollama(&self, messages: &[Message]) -> Result<String> {
        let req = OllamaChatRequest {
            model:    self.llm.model.clone(),
            messages: messages.to_vec(),
            stream:   false,
            options:  OllamaOptions { num_predict: self.llm.max_tokens, temperature: 0.3 },
        };

        let resp = self.http.post(&self.llm.api_url).json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama error {}: {}", status, body));
        }

        let data: OllamaChatResponse = resp.json().await?;
        Ok(data.message.content)
    }

    async fn call_openai_compatible(&self, messages: &[Message]) -> Result<String> {
        let req = ChatRequest {
            model:       self.llm.model.clone(),
            messages:    messages.to_vec(),
            max_tokens:  self.llm.max_tokens,
            temperature: 0.3,
            stream:      false,
        };

        let mut builder = self.http.post(&self.llm.api_url).json(&req);
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

    async fn call_gemini(&self, system_prompt: &str, messages: &[Message]) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.llm.model, self.llm.api_key
        );

        // Konversi messages ke format Gemini — skip system (sudah di system_instruction)
        let contents: Vec<GeminiContent> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| GeminiContent {
                role:  if m.role == "assistant" { "model".to_string() } else { "user".to_string() },
                parts: vec![GeminiPart { text: m.content.clone() }],
            })
            .collect();

        if contents.is_empty() {
            return Err(anyhow::anyhow!("No messages to send to Gemini"));
        }

        let req = GeminiRequest {
            system_instruction: GeminiSystemInstruction {
                parts: vec![GeminiPart { text: system_prompt.to_string() }],
            },
            contents,
            generation_config: GeminiGenerationConfig {
                max_output_tokens: self.llm.max_tokens,
                temperature:       0.3,
            },
        };

        let resp = self.http.post(&url).json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body   = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini API error {}: {}", status, body));
        }

        let data: GeminiResponse = resp.json().await?;
        data.candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| anyhow::anyhow!("Gemini returned empty response"))
    }
}

// ══════════════════════════════════════════════════════════════════
//  PROMPT BUILDERS
// ══════════════════════════════════════════════════════════════════

fn build_system_prompt() -> String {
    "Kamu adalah Asisten Akademik PMPSTI (Program Magister Teknik Sistem Informasi) \
     Universitas Gadjah Mada yang pintar dan ramah. Gunakan Markdown untuk format jawaban: \
     **bold** untuk istilah penting, tabel Markdown untuk data tabular, dan numbered list untuk urutan.\
     \n\nAturan:\n\
     1. Untuk pertanyaan seputar akademik, kurikulum, dokumen kampus, atau informasi PMPSTI: \
        jawab berdasarkan teks Konteks yang diberikan, dan sebutkan sumber dengan format [nomor].\n\
     2. Jika informasi spesifik tidak ada di Konteks, sampaikan bahwa informasi tersebut tidak \
        tersedia dalam dokumen yang diindeks, lalu sarankan pengguna menghubungi pihak program \
        studi secara langsung.\n\
     3. Untuk pertanyaan casual atau sapaan (contoh: apa kabar, halo, siapa kamu), jawab secara \
        natural dan ramah sebagai asisten akademik — tidak perlu merujuk dokumen.\n\
     4. Jika konteks berisi daftar atau tabel (misal daftar mitra kerjasama, jadwal, mata kuliah): \
        WAJIB sebutkan SEMUA entri tanpa pengecualian dalam bentuk tabel Markdown dengan kolom yang sesuai. \
        Hitung dan sebutkan totalnya di akhir.\n\
     5. Jangan mengarang fakta akademik yang tidak ada di konteks.\n\
     6. Gunakan bahasa Indonesia yang baik dan jelas."
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
