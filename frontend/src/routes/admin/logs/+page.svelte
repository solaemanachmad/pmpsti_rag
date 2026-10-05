<script lang="ts">
  import { onMount } from 'svelte';
  import { admin } from '$lib/api/client';
  import { Loader2, RefreshCw, ChevronLeft, ChevronRight } from 'lucide-svelte';

  interface QueryLog {
    id: number; query_text: string; detected_language: string;
    num_results: number; search_time_ms: number;
    user_id: number | null; session_id: string | null; created_at: string;
  }

  let logs: QueryLog[] = [];
  let total = 0;
  let page = 0;
  const PER_PAGE = 50;
  let loading = false;
  let error = '';

  onMount(() => loadLogs(0));

  async function loadLogs(p: number) {
    loading = true; error = ''; page = p;
    try {
      const r = await admin.queryLogs(PER_PAGE, p * PER_PAGE);
      logs = r.logs; total = r.total;
    }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  $: totalPages = Math.ceil(total / PER_PAGE);
</script>

<svelte:head><title>Log Query — Admin PMPSTI</title></svelte:head>

<div class="mb-6 flex items-center justify-between gap-3 flex-wrap">
  <div>
    <h1 class="text-xl font-bold">Log Query</h1>
    <p class="text-sm text-muted-foreground mt-0.5">
      {total.toLocaleString()} total query · halaman {page + 1} dari {totalPages || 1}
    </p>
  </div>
  <button on:click={() => loadLogs(page)}
    class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground border rounded-lg px-3 py-2 transition-colors">
    <RefreshCw size={13} /> Refresh
  </button>
</div>

{#if loading}
  <div class="flex items-center justify-center py-12 text-muted-foreground gap-2">
    <Loader2 size={18} class="animate-spin" /> Memuat...
  </div>
{:else if error}
  <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{error}</div>
{:else}
  <div class="bg-card border rounded-xl overflow-hidden mb-4">
    <table class="w-full text-sm">
      <thead class="bg-muted/50 text-muted-foreground text-xs uppercase tracking-wide">
        <tr>
          <th class="px-4 py-2.5 text-left">Query</th>
          <th class="px-4 py-2.5 text-right hidden sm:table-cell">Hasil</th>
          <th class="px-4 py-2.5 text-right hidden sm:table-cell">Waktu</th>
          <th class="px-4 py-2.5 text-right hidden md:table-cell">Tanggal</th>
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each logs as log}
          <tr class="hover:bg-muted/30 transition-colors">
            <td class="px-4 py-3">
              <p class="leading-relaxed text-foreground">{log.query_text}</p>
              <span class="text-[10px] text-muted-foreground/60 uppercase tracking-wide">{log.detected_language}</span>
            </td>
            <td class="px-4 py-3 text-right hidden sm:table-cell">
              <span class="text-xs font-mono {log.num_results === 0 ? 'text-rose-500' : 'text-emerald-600'}">
                {log.num_results}
              </span>
            </td>
            <td class="px-4 py-3 text-right hidden sm:table-cell text-xs font-mono text-muted-foreground">
              {log.search_time_ms}ms
            </td>
            <td class="px-4 py-3 text-right hidden md:table-cell text-xs text-muted-foreground">
              {log.created_at?.slice(0,16).replace('T',' ') ?? '—'}
            </td>
          </tr>
        {/each}
        {#if logs.length === 0}
          <tr><td colspan="4" class="px-4 py-8 text-center text-muted-foreground text-sm">Belum ada log.</td></tr>
        {/if}
      </tbody>
    </table>
  </div>

  {#if totalPages > 1}
    <div class="flex items-center justify-center gap-3">
      <button disabled={page === 0} on:click={() => loadLogs(page - 1)}
        class="p-2 rounded-lg border hover:bg-muted disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
        <ChevronLeft size={16} />
      </button>
      <span class="text-sm text-muted-foreground">{page + 1} / {totalPages}</span>
      <button disabled={page >= totalPages - 1} on:click={() => loadLogs(page + 1)}
        class="p-2 rounded-lg border hover:bg-muted disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
        <ChevronRight size={16} />
      </button>
    </div>
  {/if}
{/if}
