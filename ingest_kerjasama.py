#!/usr/bin/env python3
"""
Ingest data kerjasama PMPSTI langsung dari CSV ke RAG backend.
Jalankan setelah deploy:
  python ingest_kerjasama.py --token <JWT_ADMIN_TOKEN>
"""
import csv, json, sys, argparse, urllib.request, urllib.error, io

CSV_PATH   = "appendix/data/01_dokumen_kerja_sama_pmpsti.csv"
API_BASE   = "https://pmpsti-rag.onrender.com"   # ganti jika lokal
ENDPOINT   = f"{API_BASE}/api/admin/documents/ingest-text"
SOURCE_URL = "https://mtiugm.github.io/panduan_pmpsti/appendix/a_cooperation.html"

def build_content(rows):
    """Format seluruh tabel CSV jadi teks terstruktur untuk diindeks."""
    lines = [
        "Daftar Kerja Sama PMPSTI FT UGM",
        f"Total: {len(rows)} dokumen kerja sama",
        "",
    ]
    for i, r in enumerate(rows, 1):
        judul    = r.get("Judul Kerja Sama", "").strip()
        bidang   = r.get("Bidang", "").strip() or "—"
        mitra    = r.get("Mitra", "").strip()
        periode  = r.get("Periode (masa berlaku)", "").strip()
        tingkat  = r.get("Tingkat Kerja Sama", "").strip()
        lines.append(f"{i}. {judul}")
        lines.append(f"   Mitra: {mitra}")
        lines.append(f"   Bidang: {bidang}")
        lines.append(f"   Periode: {periode}")
        lines.append(f"   Tingkat: {tingkat}")
        lines.append("")
    return "\n".join(lines)

def post_json(url, payload, token):
    data = json.dumps(payload).encode()
    req  = urllib.request.Request(url, data=data, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        print(f"HTTP {e.code}: {body}", file=sys.stderr)
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--token",  required=True, help="JWT token admin")
    parser.add_argument("--api",    default=API_BASE, help="Base URL API")
    parser.add_argument("--csv",    default=CSV_PATH, help="Path ke CSV")
    args = parser.parse_args()

    endpoint = f"{args.api}/api/admin/documents/ingest-text"

    with open(args.csv, encoding="utf-8-sig") as f:
        rows = list(csv.DictReader(f))

    print(f"Loaded {len(rows)} rows dari {args.csv}")
    content = build_content(rows)
    print("Preview (200 chars):", content[:200])
    print("...")

    payload = {
        "title":       "Daftar Kerja Sama PMPSTI FT UGM",
        "content":     content,
        "source_url":  SOURCE_URL,
        "category":    "Kerjasama",
        "subcategory": "Dokumen Kerjasama",
    }

    print(f"\nPOST ke {endpoint} ...")
    result = post_json(endpoint, payload, args.token)
    print("Sukses:", json.dumps(result, indent=2, ensure_ascii=False))

if __name__ == "__main__":
    main()
