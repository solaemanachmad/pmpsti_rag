<script lang="ts">
  import { onMount } from 'svelte';
  import { admin } from '$lib/api/client';
  import type { AdminDocument } from '$lib/api/client';
  import { Loader2, Trash2, RefreshCw, Globe, Plus, X, CheckCircle2, AlertCircle } from 'lucide-svelte';

  let documents: AdminDocument[] = [];
  let loading = false;
  let error = '';

  let showIngest = false;
  let ingestUrl = ''; let ingestTitle = ''; let ingestCategory = ''; let ingestSubcategory = '';
  let ingestLoading = false; let ingestError = ''; let ingestSuccess = '';

  onMount(() => loadDocs());

  async function loadDocs() {
    loading = true; error = '';
    try { documents = await admin.documents(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
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
      });
      ingestSuccess = `Berhasil: "${res.title}" — ${res.chunks} chunk diindeks`;
      ingestUrl = ''; ingestTitle = ''; ingestCategory = ''; ingestSubcategory = '';
      showIngest = false;
      documents = []; loadDocs();
    } catch (e: unknown) { ingestError = e instanceof Error ? e.message : String(e); }
    finally { ingestLoading = false; }
  }

  async function deleteDoc(id: string) {
    if (!confirm('Hapus dokumen ini? Semua chunk terkait akan dihapus.')) return;
    try { await admin.deleteDocument(id); documents = documents.filter(d => d.id !== id); }
    catch (e) { alert('Gagal: ' + e); }
  }
</script>

<svelte:head><title>Dokumen — Admin PMPSTI</title></svelte:head>

<div class="mb-6 flex items-center justify-between gap-3 flex-wrap">
  <div>
    <h1 class="text-xl font-bold">Dokumen</h1>
    <p class="text-sm text-muted-foreground mt-0.5">{documents.length} dokumen terindeks</p>
  </div>
  <div class="flex items-center gap-2">
    <button on:click={() => { documents = []; loadDocs(); }}
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
          <th class="px-4 py-2.5 text-left">Judul / Sumber</th>
          <th class="px-4 py-2.5 text-left hidden sm:table-cell">Kategori</th>
          <th class="px-4 py-2.5 text-left hidden md:table-cell">Chunk</th>
          <th class="px-4 py-2.5 text-right">Aksi</th>
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each documents as doc}
          <tr class="hover:bg-muted/30 transition-colors">
            <td class="px-4 py-3">
              <div class="font-medium line-clamp-1">{doc.title || doc.id}</div>
              {#if doc.source_url}
                <a href={doc.source_url} target="_blank" rel="noopener noreferrer"
                   class="text-xs text-[#0055A5] hover:underline line-clamp-1">{doc.source_url}</a>
              {/if}
            </td>
            <td class="px-4 py-3 hidden sm:table-cell text-xs text-muted-foreground">{doc.category}</td>
            <td class="px-4 py-3 hidden md:table-cell text-xs font-mono text-muted-foreground">{doc.chunk_count ?? '—'}</td>
            <td class="px-4 py-3 text-right">
              <button on:click={() => deleteDoc(doc.id)}
                class="p-1.5 rounded-lg text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors">
                <Trash2 size={15} />
              </button>
            </td>
          </tr>
        {/each}
        {#if documents.length === 0}
          <tr><td colspan="4" class="px-4 py-8 text-center text-muted-foreground text-sm">Belum ada dokumen.</td></tr>
        {/if}
      </tbody>
    </table>
  </div>
{/if}
