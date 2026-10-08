<script lang="ts">
  import { admin } from '$lib/api/client';
  import type { AdminDocument, AdminChunk } from '$lib/api/client';
  import { Loader2, Trash2, RefreshCw, Globe, Plus, X, CheckCircle2,
           AlertCircle, ChevronDown, ChevronRight, Pencil, Check } from 'lucide-svelte';

  let documents: AdminDocument[] = [];
  let loading = false;
  let error = '';

  // Ingest form
  let showIngest = false;
  let ingestUrl = ''; let ingestTitle = ''; let ingestCategory = ''; let ingestSubcategory = '';
  let ingestChunkStrategy = 'auto';
  let ingestLoading = false; let ingestError = ''; let ingestSuccess = '';

  const chunkStrategyOptions = [
    { value: 'auto',       label: 'Otomatis',   desc: 'Deteksi otomatis berdasarkan konten (disarankan)' },
    { value: 'sentence',   label: 'Sentence',    desc: 'Cocok untuk artikel, prosa — 0 Gemini quota saat chunking' },
    { value: 'structural', label: 'Structural',  desc: 'Cocok untuk daftar bernomor, tabel, data dosen' },
    { value: 'semantic',   label: 'Semantic',    desc: 'Paling akurat, menggunakan Gemini quota lebih banyak' },
  ];

  // Chunk drawer — map document_id → chunk list
  let expandedDoc: string | null = null;
  let chunksByDoc: Record<string, AdminChunk[]> = {};
  let chunksLoading: Record<string, boolean> = {};
  let chunksError: Record<string, string> = {};

  // Edit document metadata
  let editingDoc: string | null = null;
  let editTitle = ''; let editCategory = ''; let editSubcategory = '';
  let savingDoc = false;

  // Load on mount via reactive
  let _loaded = false;
  $: if (!_loaded) { _loaded = true; loadDocs(); }

  async function loadDocs() {
    loading = true; error = '';
    try { documents = await admin.documents(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  async function toggleChunks(docId: string) {
    if (expandedDoc === docId) { expandedDoc = null; return; }
    expandedDoc = docId;
    if (chunksByDoc[docId]) return; // already loaded
    chunksLoading = { ...chunksLoading, [docId]: true };
    chunksError = { ...chunksError, [docId]: '' };
    try {
      chunksByDoc = { ...chunksByDoc, [docId]: await admin.listChunks(docId) };
    } catch (e: unknown) {
      chunksError = { ...chunksError, [docId]: e instanceof Error ? e.message : String(e) };
    } finally {
      chunksLoading = { ...chunksLoading, [docId]: false };
    }
  }

  async function deleteDoc(id: string) {
    if (!confirm('Hapus dokumen ini? Semua chunk terkait akan dihapus.')) return;
    try {
      await admin.deleteDocument(id);
      await loadDocs();
      if (expandedDoc === id) expandedDoc = null;
    } catch (e) { alert('Gagal: ' + e); }
  }

  function startEditDoc(doc: AdminDocument) {
    editingDoc = doc.document_id;
    editTitle = doc.title;
    editCategory = doc.category;
    editSubcategory = doc.subcategory ?? '';
  }

  function cancelEditDoc() { editingDoc = null; }

  async function saveDoc() {
    if (!editingDoc) return;
    savingDoc = true;
    try {
      await admin.updateDocument(editingDoc, {
        title: editTitle.trim(),
        category: editCategory.trim(),
        subcategory: editSubcategory.trim(),
      });
      // Update local state
      documents = documents.map(d =>
        d.document_id === editingDoc
          ? { ...d, title: editTitle.trim(), category: editCategory.trim(), subcategory: editSubcategory.trim() }
          : d
      );
      // Clear cached chunks so they reload with new metadata
      const { [editingDoc]: _, ...rest } = chunksByDoc;
      chunksByDoc = rest;
      editingDoc = null;
    } catch (e) { alert('Gagal simpan: ' + e); }
    finally { savingDoc = false; }
  }

  async function doIngest() {
    if (!ingestUrl.trim()) { ingestError = 'URL wajib diisi'; return; }
    ingestLoading = true; ingestError = ''; ingestSuccess = '';
    try {
      const res = await admin.ingestUrl({
        url: ingestUrl.trim(),
        title: ingestTitle.trim() || undefined,
        category: ingestCategory.trim() || undefined,
        subcategory: ingestSubcategory.trim() || undefined,
        chunk_strategy: ingestChunkStrategy,
      });
      ingestSuccess = `Berhasil: "${res.title}" — ${res.chunks} chunk diindeks`;
      ingestUrl = ''; ingestTitle = ''; ingestCategory = ''; ingestSubcategory = '';
      ingestChunkStrategy = 'auto';
      showIngest = false;
      await loadDocs();
    } catch (e: unknown) { ingestError = e instanceof Error ? e.message : String(e); }
    finally { ingestLoading = false; }
  }
</script>

<svelte:head><title>Dokumen — Admin PMPSTI</title></svelte:head>

<div class="mb-6 flex items-center justify-between gap-3 flex-wrap">
  <div>
    <h1 class="text-xl font-bold">Dokumen</h1>
    <p class="text-sm text-muted-foreground mt-0.5">{documents.length} dokumen terindeks</p>
  </div>
  <div class="flex items-center gap-2">
    <button on:click={() => { documents = []; chunksByDoc = {}; loadDocs(); }}
      class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground border rounded-lg px-3 py-2 transition-colors">
      <RefreshCw size={13} /> Refresh
    </button>
    <button on:click={() => { showIngest = !showIngest; ingestError = ''; ingestSuccess = ''; }}
      class="flex items-center gap-1.5 text-sm text-white bg-[#002147] hover:bg-[#003580] px-3 py-2 rounded-lg transition-colors">
      <Globe size={13} /> Tambah URL
    </button>
  </div>
</div>

{#if ingestSuccess}
  <div class="flex items-center gap-2 bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700 dark:text-emerald-400
              border border-emerald-200 dark:border-emerald-800 rounded-lg px-4 py-2.5 text-sm mb-4">
    <CheckCircle2 size={15} />{ingestSuccess}
  </div>
{/if}

{#if showIngest}
  <div class="bg-card border rounded-xl p-5 mb-5">
    <div class="flex items-center justify-between mb-4">
      <h3 class="font-semibold text-sm">Tambah Sumber dari URL</h3>
      <button on:click={() => showIngest = false} class="text-muted-foreground hover:text-foreground"><X size={15} /></button>
    </div>
    <div class="grid gap-3">
      <div>
        <label class="block text-xs font-medium text-muted-foreground mb-1">URL <span class="text-rose-500">*</span></label>
        <input bind:value={ingestUrl} placeholder="https://example.com/halaman"
          class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
      </div>
      <div class="grid sm:grid-cols-3 gap-3">
        <div>
          <label class="block text-xs font-medium text-muted-foreground mb-1">Judul</label>
          <input bind:value={ingestTitle} placeholder="Judul dokumen (opsional)"
            class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
        </div>
        <div>
          <label class="block text-xs font-medium text-muted-foreground mb-1">Kategori</label>
          <input bind:value={ingestCategory} placeholder="misal: Akademik"
            class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
        </div>
        <div>
          <label class="block text-xs font-medium text-muted-foreground mb-1">Sub-kategori</label>
          <input bind:value={ingestSubcategory} placeholder="misal: Kurikulum"
            class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
        </div>
      </div>
      <!-- Chunk strategy selector -->
      <div>
        <label class="block text-xs font-medium text-muted-foreground mb-1.5">Strategi Chunking</label>
        <div class="grid grid-cols-2 sm:grid-cols-5 gap-2">
          {#each chunkStrategyOptions as opt}
            <button type="button"
              on:click={() => ingestChunkStrategy = opt.value}
              title={opt.desc}
              class="flex flex-col items-center gap-0.5 px-2 py-2 rounded-lg border text-xs transition-colors
                     {ingestChunkStrategy === opt.value
                       ? 'border-[#0055A5] bg-[#0055A5]/10 text-[#0055A5] font-semibold'
                       : 'border-border text-muted-foreground hover:border-[#0055A5]/50 hover:text-foreground'}">
              <span class="font-medium">{opt.label}</span>
            </button>
          {/each}
        </div>
        <p class="text-xs text-muted-foreground mt-1.5">
          {chunkStrategyOptions.find(o => o.value === ingestChunkStrategy)?.desc ?? ''}
        </p>
      </div>
      {#if ingestError}
        <div class="flex items-center gap-2 text-rose-600 text-sm"><AlertCircle size={14} />{ingestError}</div>
      {/if}
      <div class="flex justify-end">
        <button on:click={doIngest} disabled={ingestLoading}
          class="flex items-center gap-2 text-sm text-white bg-[#0055A5] hover:bg-[#003580]
                 disabled:opacity-50 px-4 py-2 rounded-lg transition-colors">
          {#if ingestLoading}<Loader2 size={14} class="animate-spin" />Memproses...{:else}<Plus size={14} />Proses & Indeks{/if}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if loading}
  <div class="flex items-center justify-center py-12 text-muted-foreground gap-2">
    <Loader2 size={18} class="animate-spin" /> Memuat...
  </div>
{:else if error}
  <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{error}</div>
{:else}
  <div class="bg-card border rounded-xl overflow-hidden">
    <table class="w-full text-sm">
      <thead class="bg-muted/50 text-muted-foreground text-xs uppercase tracking-wide">
        <tr>
          <th class="px-4 py-2.5 text-left w-6"></th>
          <th class="px-4 py-2.5 text-left">Judul / Sumber</th>
          <th class="px-4 py-2.5 text-left hidden sm:table-cell">Kategori</th>
          <th class="px-4 py-2.5 text-left hidden md:table-cell">Chunk</th>
          <th class="px-4 py-2.5 text-right">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#each documents as doc (doc.document_id)}
          <!-- Document row -->
          <tr class="border-t hover:bg-muted/30 transition-colors cursor-pointer"
              on:click={() => editingDoc !== doc.document_id && toggleChunks(doc.document_id)}>
            <td class="px-3 py-3 text-muted-foreground">
              {#if expandedDoc === doc.document_id}
                <ChevronDown size={14} />
              {:else}
                <ChevronRight size={14} />
              {/if}
            </td>
            <td class="px-4 py-3">
              <div class="font-medium line-clamp-1">{doc.title || doc.document_id}</div>
              {#if doc.source_url}
                <a href={doc.source_url} target="_blank" rel="noopener noreferrer"
                   on:click|stopPropagation
                   class="text-xs text-[#0055A5] hover:underline line-clamp-1">{doc.source_url}</a>
              {/if}
            </td>
            <td class="px-4 py-3 hidden sm:table-cell text-xs text-muted-foreground">
              {doc.category}{doc.subcategory ? ` / ${doc.subcategory}` : ''}
            </td>
            <td class="px-4 py-3 hidden md:table-cell text-xs font-mono text-muted-foreground">{doc.chunk_count ?? '—'}</td>
            <td class="px-4 py-3 text-right">
              <div class="flex items-center justify-end gap-1">
                <button on:click|stopPropagation={() => startEditDoc(doc)}
                  class="p-1.5 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                  title="Edit metadata">
                  <Pencil size={14} />
                </button>
                <button on:click|stopPropagation={() => deleteDoc(doc.document_id)}
                  class="p-1.5 rounded-lg text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors"
                  title="Hapus dokumen">
                  <Trash2 size={15} />
                </button>
              </div>
            </td>
          </tr>

          <!-- Edit metadata form -->
          {#if editingDoc === doc.document_id}
            <tr class="border-t bg-blue-50/50 dark:bg-blue-900/10">
              <td colspan="5" class="px-4 py-4">
                <div class="flex items-center gap-2 mb-3">
                  <Pencil size={14} class="text-[#0055A5]" />
                  <span class="text-sm font-semibold text-[#0055A5]">Edit Metadata Dokumen</span>
                </div>
                <div class="grid sm:grid-cols-3 gap-3 mb-3">
                  <div class="sm:col-span-3">
                    <label class="block text-xs font-medium text-muted-foreground mb-1">Judul</label>
                    <input bind:value={editTitle} placeholder="Judul dokumen"
                      on:click|stopPropagation
                      class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">Kategori</label>
                    <input bind:value={editCategory} placeholder="misal: Akademik"
                      on:click|stopPropagation
                      class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
                  </div>
                  <div>
                    <label class="block text-xs font-medium text-muted-foreground mb-1">Sub-kategori</label>
                    <input bind:value={editSubcategory} placeholder="misal: Kurikulum"
                      on:click|stopPropagation
                      class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
                  </div>
                  <div class="flex items-end justify-end gap-2">
                    <button on:click|stopPropagation={cancelEditDoc}
                      class="text-sm px-3 py-2 border rounded-lg hover:bg-muted transition-colors">
                      Batal
                    </button>
                    <button on:click|stopPropagation={saveDoc} disabled={savingDoc}
                      class="flex items-center gap-1.5 text-sm px-3 py-2 bg-[#0055A5] text-white rounded-lg
                             hover:bg-[#003580] disabled:opacity-50 transition-colors">
                      {#if savingDoc}<Loader2 size={13} class="animate-spin" />{:else}<Check size={13} />{/if}
                      Simpan
                    </button>
                  </div>
                </div>
              </td>
            </tr>
          {/if}

          <!-- Chunk drawer -->
          {#if expandedDoc === doc.document_id}
            <tr class="border-t bg-muted/20">
              <td colspan="5" class="px-4 py-3">
                {#if chunksLoading[doc.document_id]}
                  <div class="flex items-center gap-2 text-sm text-muted-foreground py-2">
                    <Loader2 size={14} class="animate-spin" /> Memuat chunk...
                  </div>
                {:else if chunksError[doc.document_id]}
                  <p class="text-sm text-rose-500">{chunksError[doc.document_id]}</p>
                {:else if (chunksByDoc[doc.document_id] ?? []).length === 0}
                  <p class="text-sm text-muted-foreground">Tidak ada chunk.</p>
                {:else}
                  <div class="space-y-2">
                    {#each chunksByDoc[doc.document_id] as chunk (chunk.id)}
                      <div class="border rounded-lg bg-card p-3">
                        <div class="flex items-center justify-between mb-1.5 gap-2">
                          <span class="text-xs font-mono text-muted-foreground">
                            Chunk #{chunk.chunk_index} · ID {chunk.id}
                          </span>
                        </div>
                        <p class="text-xs text-foreground/80 leading-relaxed line-clamp-4 whitespace-pre-wrap">
                          {chunk.content}
                        </p>
                      </div>
                    {/each}
                  </div>
                {/if}
              </td>
            </tr>
          {/if}
        {/each}
        {#if documents.length === 0}
          <tr>
            <td colspan="5" class="px-4 py-8 text-center text-muted-foreground text-sm">
              Belum ada dokumen.
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
  </div>
{/if}
