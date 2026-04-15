<script lang="ts">
  import { onMount } from 'svelte';
  import { admin } from '$lib/api/client';
  import type { QueryLogStats } from '$lib/api/client';
  import { BarChart3, TrendingUp, Search, Clock, AlertCircle, Loader2 } from 'lucide-svelte';

  let stats: QueryLogStats | null = null;
  let loading = true;
  let error = '';

  onMount(async () => {
    try {
      stats = await admin.stats();
    } catch (e: any) {
      error = e.message ?? 'Gagal memuat statistik';
    } finally {
      loading = false;
    }
  });

  function pct(val: number, total: number) {
    if (!total) return 0;
    return Math.round((val / total) * 100);
  }
</script>

<svelte:head><title>Dashboard — PMPSTI RAG</title></svelte:head>

<div class="h-full overflow-y-auto p-6">
  <div class="max-w-5xl mx-auto">
    <div class="mb-6">
      <h1 class="text-xl font-semibold">Dashboard</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Statistik penggunaan query RAG</p>
    </div>

    {#if loading}
      <div class="flex items-center justify-center py-20 text-muted-foreground">
        <Loader2 size={20} class="animate-spin mr-2" />
        <span class="text-sm">Memuat statistik...</span>
      </div>

    {:else if error}
      <div class="flex items-center gap-3 bg-destructive/10 border border-destructive/20 text-destructive rounded-xl px-4 py-3">
        <AlertCircle size={16} />
        <span class="text-sm">{error}</span>
      </div>

    {:else if stats}
      <!-- Stat cards -->
      <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div class="bg-card border rounded-xl p-4">
          <div class="flex items-center gap-2 text-muted-foreground mb-2">
            <Search size={14} />
            <span class="text-xs font-medium uppercase tracking-wide">Total Query</span>
          </div>
          <p class="text-2xl font-semibold">{stats.total_queries.toLocaleString()}</p>
        </div>

        <div class="bg-card border rounded-xl p-4">
          <div class="flex items-center gap-2 text-muted-foreground mb-2">
            <TrendingUp size={14} />
            <span class="text-xs font-medium uppercase tracking-wide">Query Unik</span>
          </div>
          <p class="text-2xl font-semibold">{stats.unique_queries.toLocaleString()}</p>
        </div>

        <div class="bg-card border rounded-xl p-4">
          <div class="flex items-center gap-2 text-muted-foreground mb-2">
            <Clock size={14} />
            <span class="text-xs font-medium uppercase tracking-wide">Rata-rata Waktu</span>
          </div>
          <p class="text-2xl font-semibold">{Math.round(stats.avg_search_time_ms)} <span class="text-sm font-normal text-muted-foreground">ms</span></p>
        </div>

        <div class="bg-card border rounded-xl p-4">
          <div class="flex items-center gap-2 text-muted-foreground mb-2">
            <AlertCircle size={14} />
            <span class="text-xs font-medium uppercase tracking-wide">Nol Hasil</span>
          </div>
          <p class="text-2xl font-semibold">
            {stats.zero_result_queries}
            <span class="text-sm font-normal text-muted-foreground">
              ({pct(stats.zero_result_queries, stats.total_queries)}%)
            </span>
          </p>
        </div>
      </div>

      <!-- Charts row -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
        <!-- Queries per day -->
        <div class="bg-card border rounded-xl p-4">
          <h2 class="text-sm font-medium mb-4">Query per hari (30 hari terakhir)</h2>
          {#if stats.queries_per_day.length === 0}
            <p class="text-sm text-muted-foreground py-4 text-center">Belum ada data</p>
          {:else}
            {@const maxVal = Math.max(...stats.queries_per_day.map(([,n]) => n), 1)}
            <div class="space-y-1.5">
              {#each stats.queries_per_day.slice(0, 10) as [day, count]}
                <div class="flex items-center gap-2 text-xs">
                  <span class="text-muted-foreground w-20 shrink-0">{day}</span>
                  <div class="flex-1 bg-muted rounded-full h-2 overflow-hidden">
                    <div
                      class="h-full bg-primary rounded-full transition-all"
                      style="width: {pct(count, maxVal)}%"
                    ></div>
                  </div>
                  <span class="w-8 text-right font-medium">{count}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Top queries -->
        <div class="bg-card border rounded-xl p-4">
          <h2 class="text-sm font-medium mb-4">Query terpopuler</h2>
          {#if stats.top_queries.length === 0}
            <p class="text-sm text-muted-foreground py-4 text-center">Belum ada data</p>
          {:else}
            <div class="space-y-2">
              {#each stats.top_queries.slice(0, 8) as [q, count], i}
                <div class="flex items-center gap-3 text-xs">
                  <span class="w-5 text-center font-medium text-muted-foreground shrink-0">{i + 1}</span>
                  <span class="flex-1 truncate" title={q}>{q}</span>
                  <span class="shrink-0 bg-muted px-1.5 py-0.5 rounded font-medium">{count}×</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

      <!-- Distribution row -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <!-- Language distribution -->
        <div class="bg-card border rounded-xl p-4">
          <h2 class="text-sm font-medium mb-4">Distribusi bahasa</h2>
          {#if stats.language_distribution.length === 0}
            <p class="text-sm text-muted-foreground py-4 text-center">Belum ada data</p>
          {:else}
            {@const total = stats.language_distribution.reduce((a, [,n]) => a + n, 0)}
            <div class="space-y-2.5">
              {#each stats.language_distribution as [lang, count]}
                <div class="flex items-center gap-2 text-xs">
                  <span class="text-muted-foreground w-12 shrink-0 capitalize">{lang || 'unknown'}</span>
                  <div class="flex-1 bg-muted rounded-full h-2 overflow-hidden">
                    <div
                      class="h-full bg-primary/70 rounded-full"
                      style="width: {pct(count, total)}%"
                    ></div>
                  </div>
                  <span class="w-12 text-right text-muted-foreground">{pct(count, total)}%</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Domain distribution -->
        <div class="bg-card border rounded-xl p-4">
          <h2 class="text-sm font-medium mb-4">Distribusi domain</h2>
          {#if stats.domain_distribution.length === 0}
            <p class="text-sm text-muted-foreground py-4 text-center">Belum ada data</p>
          {:else}
            {@const total = stats.domain_distribution.reduce((a, [,n]) => a + n, 0)}
            <div class="space-y-2.5">
              {#each stats.domain_distribution as [domain, count]}
                <div class="flex items-center gap-2 text-xs">
                  <span class="text-muted-foreground w-24 shrink-0 truncate capitalize">{domain || 'unknown'}</span>
                  <div class="flex-1 bg-muted rounded-full h-2 overflow-hidden">
                    <div
                      class="h-full bg-primary/70 rounded-full"
                      style="width: {pct(count, total)}%"
                    ></div>
                  </div>
                  <span class="w-12 text-right text-muted-foreground">{pct(count, total)}%</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>
