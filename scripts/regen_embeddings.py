#!/usr/bin/env python3
"""
regen_embeddings.py — Backfill embeddings for documents in Neon DB.

Generates embeddings for rows WHERE embedding IS NULL using the same
model the Rust backend uses: intfloat/multilingual-e5-base (768-dim).

Usage:
    # Point at Neon (get connection string from Neon console)
    DATABASE_URL="postgresql://user:pass@host/db?sslmode=require" \\
        python scripts/regen_embeddings.py

    # Point at local DB
    python scripts/regen_embeddings.py
"""
import os
import sys

import psycopg2
from langchain_huggingface import HuggingFaceEmbeddings
from pgvector.psycopg2 import register_vector

DB_URL = os.environ.get(
    "DATABASE_URL",
    "postgresql://admin:admin@localhost:5432/pmpsti_rag",
)
BATCH_SIZE = 10


def main() -> None:
    print("Loading model intfloat/multilingual-e5-base …")
    model = HuggingFaceEmbeddings(model_name="intfloat/multilingual-e5-base")
    print("Model loaded.")

    conn = psycopg2.connect(DB_URL)
    register_vector(conn)
    cur = conn.cursor()

    cur.execute(
        "SELECT id, content FROM documents WHERE embedding IS NULL ORDER BY id"
    )
    rows = cur.fetchall()
    total = len(rows)
    print(f"Found {total} documents without embeddings.\n")

    if total == 0:
        print("Nothing to do.")
        return

    for i in range(0, total, BATCH_SIZE):
        batch = rows[i : i + BATCH_SIZE]
        ids = [r[0] for r in batch]
        # E5 models require "passage: " prefix for documents
        texts = [f"passage: {r[1]}" for r in batch]

        embeddings = model.embed_documents(texts)

        for doc_id, emb in zip(ids, embeddings):
            cur.execute(
                "UPDATE documents SET embedding = %s WHERE id = %s",
                (emb, doc_id),
            )

        conn.commit()
        done = min(i + BATCH_SIZE, total)
        pct = done / total * 100
        print(f"  [{done:3d}/{total}]  {pct:5.1f}%  (ids {ids[0]}…{ids[-1]})")

    print("\nDone! All embeddings written to DB.")
    cur.close()
    conn.close()


if __name__ == "__main__":
    main()
