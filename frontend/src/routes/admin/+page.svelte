<script lang="ts">
  import { onMount } from 'svelte';
  import { admin } from '$lib/api/client';
  import type { QueryLogStats } from '$lib/api/client';
  import { BarChart3, Search, Loader2, RefreshCw } from 'lucide-svelte';

  let stats: QueryLogStats | null = null;
  let loading = false;
  let error = '';

  onMount(() => loadStats());

  async function loadStats() {
    loading = true; error = '';
    try { stats = await admin.stats(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }
</script>

<svelte:head><title>Dashboard Admin — PMPSTI</title></svelte:head>

<div class="mb-6 flex items-center justify-between">
  <div>
    <h1 class="text-xl font-bold">Dashboard</h1>
    <p class="text-sm text-muted-foreground mt-0.5">Statistik penggunaan sistem</p>
  </div>
  <button on:click={loadStats} class="flex items-center gap-1.5 text-sm text-muted-foreground
         hover:text-foreground border rounded-lg px-3 py-2 transition-colors">
    <RefreshCw size={13} /> Refresh
  </button>
</div>

{#if loading}
  <div class="flex items-center justify-center py-16 text-muted-foreground gap-2">
    <Loader2 size={18} class="animate-spin" /> Memuat statistik...
  </div>
{:else if error}
  <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{error}</div>
{:else if stats}
  <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
    {#each [
      { label: 'Total Query',   value: stats.total_queries.toLocaleString(),     color: 'text-[#0055A5]' },
      { label: 'Query Unik',    value: stats.unique_queries.toLocaleString(),     color: 'text-emerald-600' },
      { label: 'Rata-rata Hasil', value: stats.avg_results.toFixed(1),           color: 'text-amber-600' },
      { label: 'Tanpa Hasil',   value: stats.zero_result_queries.toLocaleString(), color: 'text-rose-500' },
    ] as kpi}
      <div class="bg-card border rounded-xl p-4">
        <div class="text-xs text-muted-foreground mb-1">{kpi.label}</div>
        <div class="text-2xl font-bold {kpi.color}">{kpi.value}</div>
      </div>
    {/each}
  </div>

  <div class="grid sm:grid-cols-2 gap-4 mb-4">
    <div class="bg-card border rounded-xl p-4">
      <h3 class="text-sm font-semibold mb-3 flex items-center gap-2">
        <BarChart3 size={14} class="text-[#0055A5]" /> Query 30 Hari Terakhir
      </h3>
      <div class="space-y-1.5 max-h-52 overflow-y-auto">
        {#each stats.queries_per_day.slice(0, 30) as [day, count]}
          {@const max = Math.max(...stats.queries_per_day.map(x => x[1]))}
          <div class="flex items-center gap-2 text-xs">
            <span class="text-muted-foreground w-24 shrink-0">{day}</span>
            <div class="flex-1 bg-muted rounded-full h-2 overflow-hidden">
              <div class="h-full bg-[#0055A5] rounded-full" style="width:{Math.min(100,(count/max)*100)}%"></div>
            </div>
            <span class="font-medium w-8 text-right">{count}</span>
          </div>
        {/each}
      </div>
    </div>

    <div class="bg-card border rounded-xl p-4">
      <h3 class="text-sm font-semibold mb-3 flex items-center gap-2">
        <Search size={14} class="text-[#0055A5]" /> Query Terpopuler
      </h3>
      <div class="space-y-2 max-h-52 overflow-y-auto">
        {#each stats.top_queries.slice(0, 10) as [q, count]}
          <div class="flex items-start gap-2 text-xs">
            <span class="shrink-0 px-1.5 py-0.5 rounded bg-[#0055A5]/10 text-[#0055A5] font-semibold">{count}×</span>
            <span class="text-muted-foreground leading-relaxed">{q}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <div class="bg-card border rounded-xl p-4">
    <h3 class="text-sm font-semibold mb-1">Rata-rata Waktu Pencarian</h3>
    <p class="text-2xl font-bold text-emerald-600">
      {stats.avg_search_time_ms.toFixed(0)}
      <span class="text-sm font-normal text-muted-foreground">ms</span>
    </p>
  </div>
{/if}
