#!/usr/bin/env python3
import psycopg2, json, os

LOCAL_DB = os.environ.get("LOCAL_DB", "postgresql://admin:admin@localhost:5432/pmpsti_rag")
conn = psycopg2.connect(LOCAL_DB)
cur = conn.cursor()
cur.execute('''
    SELECT document_id, chunk_index, title, source_url, content, 
           embedding::text, document_type, category, subcategory 
    FROM documents ORDER BY id
''')
rows = cur.fetchall()
print(f'Exporting {len(rows)} rows...')

data = []
for r in rows:
    data.append({
        'document_id': r[0], 'chunk_index': r[1], 'title': r[2],
        'url': r[3], 'content': r[4], 'embedding': r[5],
        'document_type': r[6], 'category': r[7], 'subcategory': r[8]
    })

out = os.path.join(os.path.dirname(__file__), 'local_export.json')
with open(out, 'w', encoding='utf-8') as f:
    json.dump(data, f, ensure_ascii=False)
print(f'Saved to {out}')
conn.close()
