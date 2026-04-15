import os
import sys
import hashlib
import re
import fitz
import pymupdf4llm
import docx
import docx.table
import docx.text.paragraph
from lxml import etree
from pathlib import Path

import psycopg2
from psycopg2.extras import execute_values
from pgvector.psycopg2 import register_vector
from langchain_text_splitters import MarkdownHeaderTextSplitter, RecursiveCharacterTextSplitter
from langchain_experimental.text_splitter import SemanticChunker
from langchain_huggingface import HuggingFaceEmbeddings

import pytesseract
from PIL import Image
import io

OCR_AVAILABLE = True
print("[INFO] OCR aktif — teks dalam gambar akan diekstrak.")

if sys.platform == 'win32':
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')

# ─── Configuration ───
BASE_DIR = Path(__file__).resolve().parent.parent / "docs"
DB_URL   = "postgresql://admin:admin@localhost:5432/pmpsti_rag"

# skip small logo
MIN_IMAGE_WIDTH  = 100
MIN_IMAGE_HEIGHT = 100

print("Loading AI Model (smultilingual-e5-base)...")
embeddings = HuggingFaceEmbeddings(model_name="intfloat/multilingual-e5-base")


# ══════════════════════════════════════════════════════════════════
#  DATABASE
# ══════════════════════════════════════════════════════════════════

def connect_db():
    conn = psycopg2.connect(DB_URL)
    cursor = conn.cursor()
    cursor.execute("CREATE EXTENSION IF NOT EXISTS vector;")
    conn.commit()
    register_vector(conn)
    return conn

def setup_database(conn):
    """Create the main documents table and vector index if they don't exist."""
    cursor = conn.cursor()
    
    cursor.execute("CREATE EXTENSION IF NOT EXISTS vector;")
    
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS documents (
            id SERIAL PRIMARY KEY,
            document_id TEXT,
            title TEXT,
            content TEXT,
            document_type TEXT,
            source_url TEXT,
            category TEXT DEFAULT '',
            subcategory TEXT DEFAULT '',
            file_hash TEXT DEFAULT '',
            embedding vector(768),
            page_number INTEGER,
            chunk_index INTEGER
        );
    """)
    
    cursor.execute("""
        CREATE INDEX IF NOT EXISTS documents_embedding_idx 
        ON documents USING hnsw (embedding vector_cosine_ops);
    """)
    
    conn.commit()
    cursor.close()
    print("[INFO] Database schema & Vector Index ready.")

def print_db_stats(conn):
    """OUTPUT_DB: Statistics of the current documents in the database."""
    cursor = conn.cursor()
    print("\n" + "="*50)
    print(" OUTPUT_DB: Statistics Docs in PostgreSQL ")
    print("="*50)
    
    cursor.execute("SELECT COUNT(*) FROM documents;")
    total_chunks = cursor.fetchone()[0]
    
    cursor.execute("SELECT COUNT(DISTINCT document_id) FROM documents;")
    total_docs = cursor.fetchone()[0]
    
    cursor.execute("SELECT category, COUNT(*) FROM documents GROUP BY category ORDER BY count DESC;")
    categories = cursor.fetchall()
    
    print(f"Total Dokumen Unik  : {total_docs} dokumen")
    print(f"Total Potongan Teks : {total_chunks} chunks")
    
    if categories:
        print("\nDistribusi per Kategori Folder:")
        for cat, count in categories:
            cat_name = cat if cat else "General"
            print(f"  - {cat_name}: {count} chunks")
            
    print("="*50 + "\n")
    cursor.close()


# ══════════════════════════════════════════════════════════════════
#  PDF & DOCX EXTRACTION
# ══════════════════════════════════════════════════════════════════
# (Bagian fungsi ekstraksi PDF dan DOCX tetap sama persis seperti kodemu sebelumnya)

def _ocr_image_bytes(img_bytes: bytes) -> str:
    if not OCR_AVAILABLE: return ""
    try:
        image = Image.open(io.BytesIO(img_bytes)).convert("RGB")
        return pytesseract.image_to_string(image, lang="ind+eng").strip()
    except Exception: return ""

def extract_pdf_images_text(filepath: Path) -> str:
    if not OCR_AVAILABLE: return ""
    doc = fitz.open(str(filepath))
    blocks = []
    for page in doc:
        for img_info in page.get_images(full=True):
            xref = img_info[0]
            base_image = doc.extract_image(xref)
            if base_image["width"] < MIN_IMAGE_WIDTH or base_image["height"] < MIN_IMAGE_HEIGHT: continue
            text = _ocr_image_bytes(base_image["image"])
            if text: blocks.append(f"\n\n\n{text}")
    doc.close()
    return "\n".join(blocks)

def extract_pdf_markdown(filepath: Path) -> str:
    try:
        doc = fitz.open(str(filepath))
        full_md_list = []
        
        for page_num in range(len(doc)):

            page_md = pymupdf4llm.to_markdown(doc, pages=[page_num], show_warning=False)
            full_md_list.append(page_md)
            
            page_obj = doc[page_num]
            for img_info in page_obj.get_images(full=True):
                xref = img_info[0]
                base_image = doc.extract_image(xref)
                if base_image["width"] < MIN_IMAGE_WIDTH or base_image["height"] < MIN_IMAGE_HEIGHT:
                    continue
                
                ocr_text = _ocr_image_bytes(base_image["image"])
                if ocr_text:
                    full_md_list.append(f"\n\n> [OCR Image Content Page {page_num+1}]:\n{ocr_text}\n")
        
        doc.close()
        return "\n".join(full_md_list)
    except Exception as e:
        print(f"  [ERROR] PDF gagal: {e}")
        return ""

def _table_to_markdown(table: docx.table.Table) -> str:
    rows = table.rows
    if not rows: return ""
    lines = []
    header_cells = [cell.text.replace("\n", " ").strip() for cell in rows[0].cells]
    lines.append("| " + " | ".join(header_cells) + " |")
    lines.append("| " + " | ".join("---" for _ in header_cells) + " |")
    for row in rows[1:]:
        cells = [cell.text.replace("\n", " ").strip() for cell in row.cells]
        lines.append("| " + " | ".join(cells) + " |")
    return "\n".join(lines)

def _get_docx_image_texts(filepath: Path) -> dict:
    if not OCR_AVAILABLE: return {}
    result = {}
    try:
        doc_obj = docx.Document(str(filepath))
        for rel in doc_obj.part.rels.values():
            if "image" in rel.reltype:
                try:
                    text = _ocr_image_bytes(rel.target_part.blob)
                    if text: result[rel.rId] = text
                except Exception: pass
    except Exception: pass
    return result

def extract_docx_markdown(filepath: Path) -> str:
    try: doc_obj = docx.Document(str(filepath))
    except Exception as e: print(f"  [ERROR] DOCX gagal: {e}"); return ""
    
    image_texts = _get_docx_image_texts(filepath)
    if image_texts: print(f"  [OCR] Ditemukan {len(image_texts)} gambar di DOCX...")
    
    lines = []
    for block in doc_obj.element.body:
        local = block.tag.split("}")[-1] if "}" in block.tag else block.tag
        if local == "p":
            para = docx.text.paragraph.Paragraph(block, doc_obj)
            text = para.text.strip()
            has_image = block.find(".//{http://schemas.openxmlformats.org/drawingml/2006/main}blip") is not None
            style = para.style.name if para.style else ""
            
            if style.startswith("Heading 1") and text: lines.append(f"\n# {text}")
            elif style.startswith("Heading 2") and text: lines.append(f"\n## {text}")
            elif style.startswith("Heading 3") and text: lines.append(f"\n### {text}")
            elif style.startswith("Heading 4") and text: lines.append(f"\n#### {text}")
            elif text: lines.append(text)
            
            if has_image and image_texts:
                for ocr_text in list(image_texts.values()):
                    lines.append(f"\n\n{ocr_text}\n")
                    image_texts = {}
                    break
        elif local == "tbl":
            md_table = _table_to_markdown(docx.table.Table(block, doc_obj))
            if md_table: lines.append("\n" + md_table + "\n")
            
    if image_texts:
        lines.append("\n## Extracted Image Text")
        for ocr_text in image_texts.values(): lines.append(ocr_text)
    return "\n\n".join(lines)


# ══════════════════════════════════════════════════════════════════
#  METADATA HELPERS
# ══════════════════════════════════════════════════════════════════

def get_category(filepath: Path, base_dir: Path) -> str:
    parts = filepath.relative_to(base_dir).parts
    return parts[0] if len(parts) > 1 else "General"

def get_subcategory(filepath: Path, base_dir: Path) -> str:
    parts = filepath.relative_to(base_dir).parts
    return parts[1] if len(parts) > 2 else ""

def file_hash(filepath: Path) -> str:
    h = hashlib.md5()
    with open(filepath, "rb") as f:
        while chunk := f.read(8192): h.update(chunk)
    return h.hexdigest()

def clean_markdown(text: str) -> str:
    lines = text.splitlines()
    from collections import Counter
    line_count = Counter(l.strip() for l in lines if len(l.strip()) < 60)
    repeated = {l for l, c in line_count.items() if c >= 3 and l}
    cleaned = [line for line in lines if line.strip() not in repeated]
    text = "\n".join(cleaned)
    text = re.sub(r'\n{3,}', '\n\n', text)
    text = re.sub(r'[\x00-\x08\x0b\x0c\x0e-\x1f\x7f\u200b\u200c\u200d\ufeff]', '', text)
    text = re.sub(r'(\w)-\n(\w)', r'\1\2', text)
    return text.strip()


# ══════════════════════════════════════════════════════════════════
#  MAIN PIPELINE
# ══════════════════════════════════════════════════════════════════

def process_all_documents(base_dir: Path):
    extensions = {".pdf", ".docx"}
    files = [
        Path(root) / fname
        for root, _, filenames in os.walk(base_dir)
        for fname in filenames
        if os.path.splitext(fname)[1].lower() in extensions
    ]

    print(f"\nDitemukan {len(files)} dokumen di {base_dir}")

    conn = connect_db()
    setup_database(conn)
    cursor = conn.cursor()

    # Splitters
    markdown_splitter = MarkdownHeaderTextSplitter(headers_to_split_on=[
        ("#", "Chapter"), ("##", "Subchapter"), ("###", "Section")
    ])
    semantic_chunker = SemanticChunker(embeddings, breakpoint_threshold_type="percentile")
    
    # Fallback splitter untuk memastikan chunk tidak melebihi ~512 token (LaBSE max)
    # 1500 karakter biasanya aman di bawah 512 token
    fallback_splitter = RecursiveCharacterTextSplitter(chunk_size=1500, chunk_overlap=150)

    for i, filepath in enumerate(files):
        rel_path  = str(filepath.relative_to(base_dir))
        doc_title = filepath.stem
        ext       = filepath.suffix.lower()

        print(f"\n[{i+1}/{len(files)}] Memproses: {rel_path}")

        # --- LOGIKA UPSERT / DELETE ---
        fhash = file_hash(filepath)
        cursor.execute("SELECT file_hash FROM documents WHERE document_id = %s LIMIT 1", (doc_title,))
        existing_row = cursor.fetchone()
        
        if existing_row:
            if existing_row[0] == fhash:
                print("  [SKIP] File sudah ada di database dan tidak ada perubahan.")
                continue
            else:
                print("  [UPDATE] Mendeteksi versi baru dokumen. Menghapus data lama dari database...")
                cursor.execute("DELETE FROM documents WHERE document_id = %s", (doc_title,))
                conn.commit()

        # --- EKSTRAKSI TEKS ---
        if ext == ".pdf":
            md_text  = extract_pdf_markdown(filepath)
            doc_type = "pdf"
        elif ext == ".docx":
            md_text  = extract_docx_markdown(filepath)
            doc_type = "docx"
        else:
            continue

        md_text = clean_markdown(md_text) 
        if len(md_text) < 50:
            print("  [WARN] Teks terlalu sedikit, file di-skip.")
            continue

        category    = get_category(filepath, base_dir)
        subcategory = get_subcategory(filepath, base_dir)

        print("  Memotong dokumen menjadi semantic chunks...")
        chapter_chunks = markdown_splitter.split_text(md_text)

        final_chunks = []
        for chunk in chapter_chunks:
            # Gunakan try-except karena semantic chunker kadang error pada teks aneh/pendek
            try:
                # Pastikan input berupa string konten, bukan objek Document jika error
                content = chunk.page_content if hasattr(chunk, 'page_content') else str(chunk)
                semantic_splits = semantic_chunker.create_documents([content])
                
                for sem_chunk in semantic_splits:
                    # Mewarisi metadata dari parent (Markdown Splitter)
                    sem_chunk.metadata.update(chunk.metadata) 
                    
                    if len(sem_chunk.page_content) > 1500:
                        sub_chunks = fallback_splitter.split_documents([sem_chunk])
                        final_chunks.extend(sub_chunks)
                    else:
                        final_chunks.append(sem_chunk)
            except Exception as e:
                print(f"  [Skip Chunk] Gagal memproses chunk: {e}")
                continue

        print(f"  Total chunks akhir: {len(final_chunks)}")

        # --- BATCH EMBEDDING & INSERT ---
        texts_to_embed = []
        db_records = []

        for chunk_index, doc in enumerate(final_chunks):
            text_content = doc.page_content.strip()
            if len(text_content) < 20: continue

            # Ambil metadata hierarki (Bab > Subbab)
            metadata = doc.metadata
            hierarchy = [str(metadata.get(h, "")) for h in ["Chapter", "Subchapter", "Section"] if metadata.get(h)]
            hierarchy_str = " > ".join(hierarchy)
            
            # Format khusus untuk model intfloat/multilingual-e5-base
            # E5 mewajibkan prefix 'passage: ' untuk dokumen yang disimpan di DB
            text_for_ai = f"passage: {doc_title} | {hierarchy_str}\n{text_content}"
            
            # Judul spesifik untuk tampilan UI nanti
            specific_title = f"{doc_title} ({hierarchy_str})" if hierarchy_str else doc_title

            texts_to_embed.append(text_for_ai)
            db_records.append([
                doc_title, specific_title, text_content, doc_type, rel_path,
                category, subcategory, fhash, None, metadata.get("page"), chunk_index
            ])

        if texts_to_embed:
            print(f"  Membuat embeddings secara batch untuk {len(texts_to_embed)} chunks...")
            # Batch embedding (jauh lebih cepat dan hemat network call)
            embedding_vectors = embeddings.embed_documents(texts_to_embed)

            # Masukkan hasil vektor ke dalam db_records
            for idx, vector in enumerate(embedding_vectors):
                db_records[idx][8] = vector 

            print("  Menyimpan ke database (Bulk Insert)...")
            insert_query = """
                INSERT INTO documents
                    (document_id, title, content, document_type, source_url,
                    category, subcategory, file_hash, embedding, page_number, chunk_index)
                VALUES %s
            """
            execute_values(cursor, insert_query, db_records)
            conn.commit()
            print(f"  [OK] Berhasil menyimpan data ke PostgreSQL.")

    # --- Print Stats Output DB ---
    print_db_stats(conn)

    cursor.close()
    conn.close()
    print("All done!")

if __name__ == "__main__":
    if not BASE_DIR.exists():
        print(f"ERROR: Folder is not found: {BASE_DIR}")
        sys.exit(1)
    process_all_documents(BASE_DIR)