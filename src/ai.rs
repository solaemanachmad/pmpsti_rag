use reqwest::Client;
use serde_json::{json, Value};
use log::{info, error};

pub struct AiClient {
    client: Client,
    api_key: String,
}

impl AiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn synthesize_answer(
        &self,
        query: &str,
        context_text: &str, // 1. UBAH TIPE DATA DI SINI (dari &[(...)] menjadi &str)
        history: Option<Vec<crate::ChatMessage>>, 
    ) -> Result<String, String> {
        info!("🤖 Merangkai ingatan dan mengirim ke AI...");

        // 2. HAPUS LOOP FOR DI BAWAH INI KARENA SUDAH DI-FORMAT OLEH main.rs
        // let mut context_text = String::new();
        // for (i, (title, content, _score)) in search_results.iter().enumerate() { ... }

        let system_prompt = "\
Anda adalah Asisten Akademik PMPSTI yang pintar dan ramah.
ATURAN KETAT:
1. Jawab pertanyaan mahasiswa HANYA berdasarkan 'Konteks Dokumen Kampus' yang diberikan pada pesan terakhir.
2. Jika jawabannya tidak ada di dokumen, katakan: 'Maaf, saya tidak menemukan informasi tersebut di panduan kampus.'
3. DILARANG mengarang jawaban di luar dokumen.
4. PENTING: Perhatikan riwayat percakapan untuk memahami konteks (misal jika mahasiswa bertanya 'bagaimana dengan biayanya?', itu merujuk pada topik sebelumnya).";

        // 3. context_text SEKARANG LANGSUNG DIPAKAI DI SINI
        let user_prompt = format!("Konteks Dokumen Kampus:\n{}\n\nPertanyaan Mahasiswa Saat Ini: {}", context_text, query);

        // --- MERANGKAI INGATAN AI ---
        let mut llm_messages = Vec::new();
        
        // 1. Karakter AI (System)
        llm_messages.push(json!({"role": "system", "content": system_prompt}));
        
        // 2. Ingatan Masa Lalu (History)
        if let Some(hist) = history {
            for msg in hist {
                llm_messages.push(json!({"role": msg.role, "content": msg.content}));
            }
        }
        
        // 3. Konteks Dokumen & Pertanyaan Detik Ini
        llm_messages.push(json!({"role": "user", "content": user_prompt}));

        let request_body = json!({
            "model": "llama-3.1-8b-instant", 
            "messages": llm_messages,
            "temperature": 0.2
        });

        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions") 
            .bearer_auth(&self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let json_resp: Value = response.json().await.map_err(|e| e.to_string())?;
            let answer = json_resp["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("Maaf, AI gagal memproses jawaban.")
                .to_string();
            
            Ok(answer)
        } else {
            let err_msg = response.text().await.unwrap_or_default();
            error!("API Error: {}", err_msg);
            Err("Gagal menghubungi server AI.".to_string())
        }
    }
}