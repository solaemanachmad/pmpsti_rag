#!/usr/bin/env python3
"""Export local PostgreSQL data and import to Neon via psycopg2."""
import psycopg2
import psycopg2.extras
import json
import sys
import os

LOCAL_DB = os.environ.get("LOCAL_DB", "postgresql://admin:admin@localhost:5432/pmpsti_rag")
NEON_DB = os.environ.get("NEON_DB", "")

if not NEON_DB:
    print("ERROR: Set NEON_DB environment variable to your Neon connection string")
    sys.exit(1)

print("Connecting to local DB...")
local_conn = psycopg2.connect(LOCAL_DB)
local_cur = local_conn.cursor()

# Cek kolom yang ada dulu
local_cur.execute("""
    SELECT column_name FROM information_schema.columns
    WHERE table_name = 'documents' ORDER BY ordinal_position
""")
cols = [r[0] for r in local_cur.fetchall()]
print(f"Columns in local DB: {cols}")

# Build query berdasarkan kolom yang ada
select_cols = []
insert_cols = []

for c in ['document_id', 'chunk_index', 'title', 'url', 'content', 'embedding', 'document_type', 'metadata']:
    if c in cols:
        if c == 'embedding':
            select_cols.append('embedding::text')
        else:
            select_cols.append(c)
        insert_cols.append(c)

print(f"Columns to migrate: {insert_cols}")

query = f"SELECT {', '.join(select_cols)} FROM documents ORDER BY id"
local_cur.execute(query)
rows = local_cur.fetchall()
print(f"Found {len(rows)} rows to migrate")
local_conn.close()

print("Connecting to Neon...")
neon_conn = psycopg2.connect(NEON_DB)
neon_cur = neon_conn.cursor()

batch_size = 20
inserted = 0
for i in range(0, len(rows), batch_size):
    batch = rows[i:i+batch_size]
    for row in batch:
        row_dict = dict(zip([c.replace('::text','') if '::' in c else c for c in insert_cols], row))
        
        # Defaults untuk kolom yang tidak ada di lokal
        doc_id = row_dict.get('document_id', '')
        chunk_idx = row_dict.get('chunk_index', 0)
        title = row_dict.get('title', None)
        url = row_dict.get('url', None)
        content = row_dict.get('content', '')
        embedding = row_dict.get('embedding', None)
        doc_type = row_dict.get('document_type', 'panduan')
        metadata = row_dict.get('metadata', {})

        neon_cur.execute("""
            INSERT INTO documents (document_id, chunk_index, title, url, content, embedding, document_type, metadata)
            VALUES (%s, %s, %s, %s, %s, %s::vector, %s, %s)
            ON CONFLICT (document_id, chunk_index) DO UPDATE SET
                title = EXCLUDED.title,
                url = EXCLUDED.url,
                content = EXCLUDED.content,
                embedding = EXCLUDED.embedding,
                document_type = EXCLUDED.document_type,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
        """, (doc_id, chunk_idx, title, url, content, embedding, doc_type,
              json.dumps(metadata) if isinstance(metadata, dict) else metadata or '{}'))
        inserted += 1

    neon_conn.commit()
    print(f"  Migrated {min(i+batch_size, len(rows))}/{len(rows)} rows...")

print(f"\nDone! {inserted} rows migrated to Neon.")
neon_conn.close()
