use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct ResendEmail<'a> {
    from:    &'a str,
    to:      Vec<&'a str>,
    subject: &'a str,
    html:    String,
}

pub async fn send_verification_email(
    api_key:    &str,
    to_email:   &str,
    verify_url: &str,
    base_url:   &str,
) -> Result<(), anyhow::Error> {
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: sans-serif; max-width: 600px; margin: 0 auto; padding: 32px;">
  <h2 style="color: #1a1a2e;">Verifikasi Email PMPSTI RAG</h2>
  <p>Terima kasih telah mendaftar. Klik tombol di bawah untuk mengaktifkan akun kamu:</p>
  <a href="{verify_url}"
     style="display:inline-block;background:#4f46e5;color:#fff;padding:12px 24px;
            border-radius:8px;text-decoration:none;font-weight:bold;margin:16px 0;">
    Verifikasi Email
  </a>
  <p style="color:#666;font-size:14px;">
    Link ini berlaku selama 24 jam.<br>
    Jika kamu tidak mendaftar, abaikan email ini.
  </p>
  <hr style="border:none;border-top:1px solid #eee;margin-top:32px;">
  <p style="color:#999;font-size:12px;">PMPSTI — {base_url}</p>
</body>
</html>
    "#, verify_url = verify_url, base_url = base_url);

    let client = Client::new();
    let payload = ResendEmail {
        from:    "PMPSTI RAG <noreply@mail.ugm.ac.id>",
        to:      vec![to_email],
        subject: "Verifikasi Email — PMPSTI RAG",
        html,
    };

    let res = client
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await?;

    if !res.status().is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("Resend error: {}", body));
    }

    Ok(())
}
