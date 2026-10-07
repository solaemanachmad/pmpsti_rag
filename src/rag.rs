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

    /// Cari chunks dengan strategi retrieval modern:
    /// 1. RRF (default) / semantic / keyword sesuai search_mode
    /// 2. List-query detection → paksa fetch semua chunks dari matching docs
    /// 3. Smart document expansion → expand doc multi-chunk yang ter-hit
    pub async fn search_chunks(
        &self,
        query:           &str,
        category_filter: Option<&str>,
        search_mode:     &str,
        top_k:           i64,
    ) -> Result<Vec<SearchResult>> {
        let initial_chunks = match search_mode {
            "semantic" => self.search.search_semantic(query, top_k, category_filter).await?,
            "keyword"  => self.search.search_keyword(query, top_k, category_filter).await?,
            "hybrid"   => self.search.search(query, top_k, category_filter).await?,
            _          => self.search.search_rrf(query, top_k, category_filter).await?,
        };

        if initial_chunks.is_empty() {
            return Ok(vec![]);
        }

        // ── List-query detection ─────────────────────────────────────
        // Kalau query minta "semua / daftar / sebutkan / berapa jumlah",
        // fetch semua chunks dari SEMUA dokumen yang ter-hit, bukan hanya top-k.
        // Ini memastikan dokumen list (kerjasama, mata kuliah, dll) selalu lengkap.
        let list_patterns = [
            "sebutkan", "semua", "daftar", "list", "berapa jumlah",
            "berapa banyak", "seluruh", "lengkap", "mitra", "semua mitra",
        ];
        let query_lower = query.to_lowercase();
        let is_list_query = list_patterns.iter().any(|p| query_lower.contains(p));

        if is_list_query {
            // Kumpulkan semua document_id yang ter-hit
            let hit_doc_ids: Vec<String> = {
                let mut seen = std::collections::HashSet::new();
                initial_chunks.iter()
                    .filter(|c| !c.document_id.is_empty())
                    .filter_map(|c| {
                        if seen.insert(c.document_id.clone()) { Some(c.document_id.clone()) }
                        else { None }
                    })
                    .collect()
            };
            // Fetch semua chunks dari dokumen-dokumen tersebut
            let mut all_chunks = self.search.db
                .fetch_all_chunks_for_documents(&hit_doc_ids)
                .await
                .unwrap_or(initial_chunks);
            // Deduplicate by (document_id, chunk_index)
            let mut seen_keys = std::collections::HashSet::new();
            all_chunks.retain(|c| seen_keys.insert((c.document_id.clone(), c.chunk_index)));
            // Sort: dokumen yang skornya tinggi di depan, lalu by chunk_index
            all_chunks.sort_by(|a, b| {
                b.score.partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.chunk_index.cmp(&b.chunk_index))
            });
            return Ok(all_chunks);
        }

        // ── Smart document expansion ─────────────────────────────────
        // Expand dokumen yang:
        // (a) 2+ chunks-nya ter-hit (dokumen sangat relevan), ATAU
        // (b) salah satu chunk-nya bukan chunk_index=0 (ada prefix yang belum diambil)
        let mut doc_hit_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut doc_has_nonzero_chunk: std::collections::HashSet<String> = std::collections::HashSet::new();
        for c in &initial_chunks {
            if !c.document_id.is_empty() {
                *doc_hit_count.entry(c.document_id.clone()).or_insert(0) += 1;
                if c.chunk_index.unwrap_or(0) > 0 {
                    doc_has_nonzero_chunk.insert(c.document_id.clone());
                }
            }
        }
        let expand_ids: Vec<String> = doc_hit_count.into_iter()
            .filter(|(id, count)| *count >= 2 || doc_has_nonzero_chunk.contains(id))
            .map(|(id, _)| id)
            .collect();

        if expand_ids.is_empty() {
            return Ok(initial_chunks);
        }

        let mut expanded = self.search.db
            .fetch_all_chunks_for_documents(&expand_ids)
            .await
            .unwrap_or_default();
        // Tambahkan chunks dari dokumen lain yang tidak di-expand
        for c in initial_chunks {
            if !expand_ids.contains(&c.document_id) {
                expanded.push(c);
            }
        }
        // Deduplicate
        let mut seen_keys = std::collections::HashSet::new();
        expanded.retain(|c| seen_keys.insert((c.document_id.clone(), c.chunk_index)));
        Ok(expanded)
    }

    /// Panggil LLM dengan chunks yang sudah difilter oleh handler.
    /// chunks = apa yang akan ditampilkan sebagai kartu sumber → nomor [1]..[N] cocok.
    /// Menerima slice of references (sesuai return type filter_sources).
    pub async fn answer_with_chunks(
        &self,
        query:   &str,
        history: &[Message],
        chunks:  &[&SearchResult],
    ) -> Result<String> {
        if chunks.is_empty() {
            return Ok(
                "Maaf, saya tidak menemukan informasi yang relevan dengan pertanyaan tersebut \
                 dalam dokumen yang tersedia.".to_string()
            );
        }

        // Format context dari references
        let context = SearchEngine::format_context_refs(chunks);
        let system_prompt = build_system_prompt();

        // Suggest klarifikasi jika hasil dari >= 3 kategori berbeda
        let categories: std::collections::HashSet<&str> = chunks.iter()
            .map(|c| c.category.as_str())  // c: &&SearchResult → works fine
            .filter(|c| !c.is_empty())
            .collect();
        let clarification_hint = if categories.len() >= 3 {
            let cat_list: Vec<&str> = categories.into_iter().collect();
            format!(
                "\n\nCATATAN: Hasil mencakup {} kategori: {}. \
                 Jika pertanyaan ambigu, jawab sebaik mungkin LALU tawarkan klarifikasi.",
                cat_list.len(), cat_list.join(", ")
            )
        } else {
            String::new()
        };

        let user_message = build_user_message(query, &context, &clarification_hint);

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

        Ok(answer)
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
     Universitas Gadjah Mada yang pintar dan ramah. \
     \n\nAturan:\n\
     1. Untuk pertanyaan seputar akademik, kurikulum, dokumen kampus, atau informasi PMPSTI: \
        jawab berdasarkan teks Konteks yang diberikan, dan sebutkan sumber dengan format [nomor].\n\
     2. Jika informasi spesifik tidak ada di Konteks, sampaikan bahwa informasi tersebut tidak \
        tersedia dalam dokumen yang diindeks, lalu sarankan pengguna menghubungi pihak program \
        studi secara langsung.\n\
     3. Untuk pertanyaan casual atau sapaan (contoh: apa kabar, halo, siapa kamu), jawab secara \
        natural dan ramah sebagai asisten akademik — tidak perlu merujuk dokumen.\n\
     4. Jika konteks berisi daftar atau tabel (misal daftar mitra, jadwal, mata kuliah), WAJIB \
        sebutkan SEMUA entri secara lengkap tanpa pengecualian — jangan meringkas atau hanya \
        menyebut sebagian. Gunakan format tabel Markdown jika memungkinkan.\n\
     5. Format jawaban menggunakan Markdown: **bold** untuk penekanan, tabel dengan | kolom |, \
        daftar dengan - atau nomor. JANGAN tulis simbol ** atau | secara literal tanpa maksud \
        pemformatan.\n\
     6. Jangan mengarang fakta akademik yang tidak ada di konteks."
        .to_string()
}
fn build_user_message(query: &str, context: &str, clarification_hint: &str) -> String {
    format!(
        "Konteks dari dokumen:\n\
         ─────────────────────\n\
         {context}\n\
         ─────────────────────\n\
         {clarification_hint}\n\
         Pertanyaan: {query}"
    )
}
