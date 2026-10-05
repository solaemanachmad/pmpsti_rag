<script lang="ts">
  import { onMount } from 'svelte';
  import { keys } from '$lib/api/client';
  import type { ApiKeyInfo, CreateApiKeyResponse } from '$lib/api/client';
  import { authReady } from '$lib/stores/auth';
  import { Plus, Trash2, Copy, Check, Key, Loader2, Eye, EyeOff } from 'lucide-svelte';

  let apiKeys: ApiKeyInfo[] = [];
  let loading = false;
  let creating = false;
  let error = '';

  // Form
  let showForm = false;
  let newName = '';
  let newRateLimit = 30;
  let newExpiresAt = '';

  // Revealed key setelah create
  let newlyCreatedKey: CreateApiKeyResponse | null = null;
  let copied = false;

  onMount(() => {
    // Tunggu auth selesai sebelum fetch (httpOnly cookie harus ada)
    const unsub = authReady.subscribe(ready => {
      if (!ready) return;
      unsub();
      loadKeys();
    });
  });

  async function loadKeys() {
    loading = true;
    try { apiKeys = await keys.list(); }
    catch { error = 'Gagal memuat API keys'; }
    finally { loading = false; }
  }

  async function createKey() {
    if (!newName.trim()) return;
    creating = true;
    error = '';
    try {
      const res = await keys.create(
        newName.trim(),
        ['search', 'read'],
        newRateLimit,
        newExpiresAt || undefined
      );
      newlyCreatedKey = res;
      showForm = false;
      newName = '';
      await loadKeys();
    } catch (e: any) {
      error = e.message ?? 'Gagal membuat API key';
    } finally {
      creating = false;
    }
  }

  async function revokeKey(id: number, name: string) {
    if (!confirm(`Nonaktifkan key "${name}"?`)) return;
    try {
      await keys.revoke(id);
      await loadKeys();
    } catch (e: any) {
      error = e.message ?? 'Gagal menonaktifkan key';
    }
  }

  async function copyKey(text: string) {
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function formatDate(iso: string | null) {
    if (!iso) return '—';
    return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'short', year: 'numeric' });
  }
</script>

<svelte:head><title>API Keys — PMPSTI</title></svelte:head>

<div class="h-full overflow-y-auto p-6">
  <div class="max-w-3xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-xl font-semibold">API Keys</h1>
        <p class="text-sm text-muted-foreground mt-0.5">Kelola akses programatik ke API</p>
      </div>
      <button on:click={() => { showForm = !showForm; newlyCreatedKey = null; }}
        class="flex items-center gap-2 bg-primary text-primary-foreground text-sm font-medium px-3 py-2 rounded-lg hover:opacity-90 transition-opacity">
        <Plus size={15} />
        Buat key
      </button>
    </div>

    {#if error}
      <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 mb-4">
        {error}
      </div>
    {/if}

    <!-- Newly created key alert -->
    {#if newlyCreatedKey}
      <div class="bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 rounded-xl p-4 mb-4">
        <div class="flex items-start gap-3">
          <Key size={16} class="text-green-600 dark:text-green-400 mt-0.5 shrink-0" />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-green-800 dark:text-green-300 mb-1">
              Key berhasil dibuat — simpan sekarang, tidak bisa ditampilkan lagi!
            </p>
            <div class="flex items-center gap-2 bg-white dark:bg-green-900/20 border border-green-200 dark:border-green-700 rounded-lg px-3 py-2">
              <code class="flex-1 text-xs font-mono text-green-800 dark:text-green-300 truncate">
                {newlyCreatedKey.key}
              </code>
              <button on:click={() => newlyCreatedKey && copyKey(newlyCreatedKey.key)}
                class="shrink-0 text-green-600 hover:text-green-800 transition-colors">
                {#if copied}<Check size={14} />{:else}<Copy size={14} />{/if}
              </button>
            </div>
          </div>
          <button on:click={() => newlyCreatedKey = null} class="text-green-500 hover:text-green-700 shrink-0">✕</button>
        </div>
      </div>
    {/if}

    <!-- Create form -->
    {#if showForm}
      <div class="border rounded-xl p-4 mb-4 bg-card">
        <h2 class="text-sm font-medium mb-3">Key baru</h2>
        <div class="space-y-3">
          <div>
            <label class="text-xs font-medium text-muted-foreground mb-1 block">Nama</label>
            <input bind:value={newName} placeholder="contoh: Production App" class="input" />
          </div>
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="text-xs font-medium text-muted-foreground mb-1 block">Rate limit (req/menit)</label>
              <input bind:value={newRateLimit} type="number" min="1" max="1000" class="input" />
            </div>
            <div>
              <label class="text-xs font-medium text-muted-foreground mb-1 block">Expired (opsional)</label>
              <input bind:value={newExpiresAt} type="date" class="input" />
            </div>
          </div>
          <div class="flex gap-2 justify-end">
            <button on:click={() => showForm = false}
              class="px-3 py-1.5 text-sm border rounded-lg hover:bg-muted transition-colors">
              Batal
            </button>
            <button on:click={createKey} disabled={creating || !newName.trim()}
              class="flex items-center gap-2 bg-primary text-primary-foreground text-sm px-3 py-1.5 rounded-lg hover:opacity-90 disabled:opacity-50 transition-opacity">
              {#if creating}<Loader2 size={14} class="animate-spin" />{/if}
              Buat
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- Keys table -->
    {#if loading}
      <div class="flex items-center justify-center py-12 text-muted-foreground">
        <Loader2 size={20} class="animate-spin mr-2" />
        <span class="text-sm">Memuat...</span>
      </div>
    {:else if apiKeys.length === 0}
      <div class="text-center py-16 border rounded-xl bg-card">
        <Key size={32} class="text-muted-foreground mx-auto mb-3 opacity-40" />
        <p class="text-sm text-muted-foreground">Belum ada API key</p>
      </div>
    {:else}
      <div class="border rounded-xl overflow-hidden bg-card">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b bg-muted/40">
              <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Nama</th>
              <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden sm:table-cell">Prefix</th>
              <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Dibuat</th>
              <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Terakhir dipakai</th>
              <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Status</th>
              <th class="px-4 py-3"></th>
            </tr>
          </thead>
          <tbody class="divide-y">
            {#each apiKeys as k (k.id)}
              <tr class="hover:bg-muted/20 transition-colors">
                <td class="px-4 py-3 font-medium">{k.name}</td>
                <td class="px-4 py-3 hidden sm:table-cell">
                  <code class="text-xs bg-muted px-1.5 py-0.5 rounded font-mono">{k.key_prefix}…</code>
                </td>
                <td class="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{formatDate(k.created_at)}</td>
                <td class="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{formatDate(k.last_used_at)}</td>
                <td class="px-4 py-3">
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {k.is_active ? 'bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300' : 'bg-muted text-muted-foreground'}">
                    {k.is_active ? 'Aktif' : 'Nonaktif'}
                  </span>
                </td>
                <td class="px-4 py-3 text-right">
                  {#if k.is_active}
                    <button on:click={() => revokeKey(k.id, k.name)}
                      class="p-1.5 text-muted-foreground hover:text-destructive rounded-md hover:bg-destructive/10 transition-colors">
                      <Trash2 size={14} />
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>

