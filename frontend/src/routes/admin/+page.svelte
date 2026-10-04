<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser } from '$lib/stores/auth';
  import { admin } from '$lib/api/client';
  import type { QueryLogStats, AdminUser, AdminDocument } from '$lib/api/client';
  import {
    BarChart3, Users, FileText, List,
    Loader2, Trash2, ShieldCheck, ShieldOff,
    RefreshCw, AlertCircle, ChevronLeft, ChevronRight,
    Search, Shield, LogOut
  } from 'lucide-svelte';

  type Tab = 'stats' | 'users' | 'documents' | 'logs';
  let activeTab: Tab = 'stats';

  // ── Stats ──
  let stats: QueryLogStats | null = null;
  let statsLoading = true;
  let statsError = '';

  // ── Users ──
  let users: AdminUser[] = [];
  let usersLoading = false;
  let usersError = '';
  let userSearch = '';

  // ── Documents ──
  let documents: AdminDocument[] = [];
  let docsLoading = false;
  let docsError = '';

  // ── Logs ──
  interface QueryLog {
    id: number; query_text: string; detected_language: string;
    num_results: number; search_time_ms: number;
    user_id: number | null; session_id: string | null; created_at: string;
  }
  let logs: QueryLog[] = [];
  let logsTotal = 0;
  let logsPage = 0;
  const LOGS_PER_PAGE = 50;
  let logsLoading = false;
  let logsError = '';

  // ── Access guard ──
  onMount(async () => {
    const unsubscribe = currentUser.subscribe(async u => {
      if (u === null) { goto('/login'); return; }
      if (u && u.role !== 'admin') { goto('/chat'); return; }
      if (u?.role === 'admin') {
        unsubscribe();
        await loadStats();
      }
    });
  });

  async function loadStats() {
    statsLoading = true; statsError = '';
    try { stats = await admin.stats(); }
    catch (e: unknown) { statsError = e instanceof Error ? e.message : String(e); }
    finally { statsLoading = false; }
  }

  async function loadUsers() {
    if (users.length && !usersError) return;
    usersLoading = true; usersError = '';
    try { users = await admin.users(); }
    catch (e: unknown) { usersError = e instanceof Error ? e.message : String(e); }
    finally { usersLoading = false; }
  }

  async function loadDocuments() {
    if (documents.length && !docsError) return;
    docsLoading = true; docsError = '';
    try { documents = await admin.documents(); }
    catch (e: unknown) { docsError = e instanceof Error ? e.message : String(e); }
    finally { docsLoading = false; }
  }

  async function loadLogs(page = 0) {
    logsLoading = true; logsError = '';
    logsPage = page;
    try {
      const r = await admin.queryLogs(LOGS_PER_PAGE, page * LOGS_PER_PAGE);
      logs = r.logs; logsTotal = r.total;
    }
    catch (e: unknown) { logsError = e instanceof Error ? e.message : String(e); }
    finally { logsLoading = false; }
  }

  async function switchTab(tab: Tab) {
    activeTab = tab;
    if (tab === 'users') loadUsers();
    if (tab === 'documents') loadDocuments();
    if (tab === 'logs') loadLogs(0);
  }

  async function toggleUser(u: AdminUser) {
    const newActive = !u.is_active;
    try {
      await admin.toggleUser(u.id, newActive);
      users = users.map(x => x.id === u.id ? { ...x, is_active: newActive } : x);
    } catch (e) { alert('Gagal: ' + e); }
  }

  async function setRole(u: AdminUser, role: string) {
    try {
      await admin.setRole(u.id, role);
      users = users.map(x => x.id === u.id ? { ...x, role } : x);
    } catch (e) { alert('Gagal: ' + e); }
  }

  async function deleteDoc(id: string) {
    if (!confirm('Hapus dokumen ini?')) return;
    try {
      await admin.deleteDocument(id);
      documents = documents.filter(d => d.id !== id);
    } catch (e) { alert('Gagal: ' + e); }
  }

  $: filteredUsers = userSearch.trim()
    ? users.filter(u =>
        u.email.toLowerCase().includes(userSearch.toLowerCase()) ||
        (u.display_name ?? '').toLowerCase().includes(userSearch.toLowerCase()))
    : users;

  $: totalLogPages = Math.ceil(logsTotal / LOGS_PER_PAGE);
</script>

<svelte:head><title>Admin — PMPSTI</title></svelte:head>

<div class="min-h-screen bg-background">
  <!-- Top bar -->
  <header class="bg-[#002147] text-white px-4 py-3 flex items-center justify-between sticky top-0 z-30 border-b border-white/10">
    <div class="flex items-center gap-3">
      <picture>
        <source srcset="/ugm-logo-white.png" media="(prefers-color-scheme: dark)" />
        <img src="/ugm-logo-white.png" alt="Logo UGM" class="w-8 h-8" />
      </picture>
      <div>
        <div class="font-semibold text-sm">Panel Admin</div>
        <div class="text-[11px] text-white/50">PMPSTI Universitas Gadjah Mada</div>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <span class="text-xs text-white/60 hidden sm:block">{$currentUser?.email ?? ''}</span>
      <a href="/chat" class="flex items-center gap-1.5 text-xs text-white/70 hover:text-white
                             bg-white/10 hover:bg-white/20 px-3 py-1.5 rounded-lg transition-colors">
        <LogOut size={13} />
        Kembali
      </a>
    </div>
  </header>

  <div class="max-w-6xl mx-auto px-4 py-6">

    <!-- Tab navigation -->
    <nav class="flex gap-1 mb-6 border-b">
      {#each ([
        { id: 'stats',     icon: BarChart3,  label: 'Dashboard' },
        { id: 'users',     icon: Users,      label: 'Pengguna' },
        { id: 'documents', icon: FileText,   label: 'Dokumen' },
        { id: 'logs',      icon: List,       label: 'Log Query' },
      ] as const)}
        <button
          on:click={() => switchTab(item.id)}
          class="flex items-center gap-2 px-4 py-2.5 text-sm font-medium transition-colors
                 border-b-2 -mb-px
                 {activeTab === item.id
                   ? 'border-[#0055A5] text-[#0055A5]'
                   : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >
          <svelte:component this={item.icon} size={15} />
          {item.label}
        </button>
      {/each}
    </nav>

    <!-- ══ STATS ══ -->
    {#if activeTab === 'stats'}
      {#if statsLoading}
        <div class="flex items-center justify-center py-16 text-muted-foreground gap-2">
          <Loader2 size={18} class="animate-spin" /> Memuat...
        </div>
      {:else if statsError}
        <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{statsError}</div>
      {:else if stats}
        <!-- KPI row -->
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 mb-6">
          {#each [
            { label: 'Total Query', value: stats.total_queries.toLocaleString(), color: 'text-[#0055A5]' },
            { label: 'Query Unik', value: stats.unique_queries.toLocaleString(), color: 'text-emerald-600' },
            { label: 'Rata-rata Hasil', value: stats.avg_results.toFixed(1), color: 'text-amber-600' },
            { label: 'Tanpa Hasil', value: stats.zero_result_queries.toLocaleString(), color: 'text-rose-500' },
          ] as kpi}
            <div class="bg-card border rounded-xl p-4">
              <div class="text-xs text-muted-foreground mb-1">{kpi.label}</div>
              <div class="text-2xl font-bold {kpi.color}">{kpi.value}</div>
            </div>
          {/each}
        </div>

        <div class="grid sm:grid-cols-2 gap-4">
          <!-- Query per hari -->
          <div class="bg-card border rounded-xl p-4">
            <h3 class="text-sm font-semibold mb-3 flex items-center gap-2">
              <BarChart3 size={14} class="text-[#0055A5]" />
              Query 30 Hari Terakhir
            </h3>
            <div class="space-y-1.5 max-h-48 overflow-y-auto">
              {#each stats.queries_per_day.slice(0, 30) as [day, count]}
                <div class="flex items-center gap-2 text-xs">
                  <span class="text-muted-foreground w-24 shrink-0">{day}</span>
                  <div class="flex-1 bg-muted rounded-full h-2 overflow-hidden">
                    <div class="h-full bg-[#0055A5] rounded-full"
                         style="width: {Math.min(100, (count / Math.max(...stats.queries_per_day.map(x => x[1]))) * 100)}%">
                    </div>
                  </div>
                  <span class="font-medium w-8 text-right">{count}</span>
                </div>
              {/each}
            </div>
          </div>

          <!-- Top queries -->
          <div class="bg-card border rounded-xl p-4">
            <h3 class="text-sm font-semibold mb-3 flex items-center gap-2">
              <Search size={14} class="text-[#0055A5]" />
              Query Terpopuler
            </h3>
            <div class="space-y-2 max-h-48 overflow-y-auto">
              {#each stats.top_queries.slice(0, 10) as [q, count]}
                <div class="flex items-start gap-2 text-xs">
                  <span class="shrink-0 px-1.5 py-0.5 rounded bg-[#0055A5]/10 text-[#0055A5] font-semibold">{count}×</span>
                  <span class="text-muted-foreground line-clamp-2 leading-relaxed">{q}</span>
                </div>
              {/each}
            </div>
          </div>
        </div>

        <div class="mt-4 bg-card border rounded-xl p-4">
          <h3 class="text-sm font-semibold mb-2">Rata-rata Waktu Pencarian</h3>
          <p class="text-2xl font-bold text-emerald-600">{stats.avg_search_time_ms.toFixed(0)} <span class="text-sm font-normal text-muted-foreground">ms</span></p>
        </div>
      {/if}

    <!-- ══ USERS ══ -->
    {:else if activeTab === 'users'}
      <div class="flex items-center gap-3 mb-4">
        <div class="relative flex-1 max-w-xs">
          <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input bind:value={userSearch} placeholder="Cari email / nama..."
                 class="w-full pl-8 pr-3 py-2 text-sm border rounded-lg bg-background outline-none
                        focus:ring-2 focus:ring-[#0055A5]/40 focus:border-[#0055A5]/40" />
        </div>
        <button on:click={() => { users = []; loadUsers(); }}
                class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground
                       border rounded-lg px-3 py-2 transition-colors">
          <RefreshCw size={13} /> Refresh
        </button>
      </div>

      {#if usersLoading}
        <div class="flex items-center justify-center py-12 text-muted-foreground gap-2">
          <Loader2 size={18} class="animate-spin" /> Memuat...
        </div>
      {:else if usersError}
        <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{usersError}</div>
      {:else}
        <div class="bg-card border rounded-xl overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-muted/50 text-muted-foreground text-xs uppercase tracking-wide">
              <tr>
                <th class="px-4 py-2.5 text-left">Pengguna</th>
                <th class="px-4 py-2.5 text-left hidden sm:table-cell">Role</th>
                <th class="px-4 py-2.5 text-left hidden md:table-cell">Status</th>
                <th class="px-4 py-2.5 text-left hidden md:table-cell">Bergabung</th>
                <th class="px-4 py-2.5 text-right">Aksi</th>
              </tr>
            </thead>
            <tbody class="divide-y">
              {#each filteredUsers as u}
                <tr class="hover:bg-muted/30 transition-colors">
                  <td class="px-4 py-3">
                    <div class="font-medium text-foreground">{u.display_name || '—'}</div>
                    <div class="text-xs text-muted-foreground">{u.email}</div>
                  </td>
                  <td class="px-4 py-3 hidden sm:table-cell">
                    <select value={u.role}
                            on:change={e => setRole(u, (e.target as HTMLSelectElement).value)}
                            class="text-xs border rounded px-2 py-1 bg-background
                                   {u.role === 'admin' ? 'text-[#0055A5] font-semibold' : ''}">
                      <option value="user">user</option>
                      <option value="admin">admin</option>
                    </select>
                  </td>
                  <td class="px-4 py-3 hidden md:table-cell">
                    <span class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium
                                 {u.is_active ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400'
                                              : 'bg-rose-100 text-rose-600 dark:bg-rose-900/30 dark:text-rose-400'}">
                      {u.is_active ? 'Aktif' : 'Nonaktif'}
                    </span>
                  </td>
                  <td class="px-4 py-3 hidden md:table-cell text-xs text-muted-foreground">
                    {u.created_at?.slice(0,10) ?? '—'}
                  </td>
                  <td class="px-4 py-3 text-right">
                    <button on:click={() => toggleUser(u)}
                            title={u.is_active ? 'Nonaktifkan' : 'Aktifkan'}
                            class="p-1.5 rounded-lg transition-colors
                                   {u.is_active
                                     ? 'text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20'
                                     : 'text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20'}">
                      {#if u.is_active}<ShieldOff size={15}/>{:else}<ShieldCheck size={15}/>{/if}
                    </button>
                  </td>
                </tr>
              {/each}
              {#if filteredUsers.length === 0}
                <tr><td colspan="5" class="px-4 py-8 text-center text-muted-foreground text-sm">
                  Tidak ada pengguna ditemukan.
                </td></tr>
              {/if}
            </tbody>
          </table>
        </div>
        <p class="text-xs text-muted-foreground mt-2">{filteredUsers.length} dari {users.length} pengguna</p>
      {/if}

    <!-- ══ DOCUMENTS ══ -->
    {:else if activeTab === 'documents'}
      <div class="flex justify-between items-center mb-4">
        <p class="text-sm text-muted-foreground">{documents.length} dokumen terindeks</p>
        <button on:click={() => { documents = []; loadDocuments(); }}
                class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground
                       border rounded-lg px-3 py-2 transition-colors">
          <RefreshCw size={13} /> Refresh
        </button>
      </div>

      {#if docsLoading}
        <div class="flex items-center justify-center py-12 text-muted-foreground gap-2">
          <Loader2 size={18} class="animate-spin" /> Memuat...
        </div>
      {:else if docsError}
        <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{docsError}</div>
      {:else}
        <div class="bg-card border rounded-xl overflow-hidden">
          <table class="w-full text-sm">
            <thead class="bg-muted/50 text-muted-foreground text-xs uppercase tracking-wide">
              <tr>
                <th class="px-4 py-2.5 text-left">Judul</th>
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
                         class="text-xs text-[#0055A5] hover:underline line-clamp-1">
                        {doc.source_url}
                      </a>
                    {/if}
                  </td>
                  <td class="px-4 py-3 hidden sm:table-cell">
                    <span class="text-xs text-muted-foreground">{doc.category}</span>
                  </td>
                  <td class="px-4 py-3 hidden md:table-cell">
                    <span class="text-xs font-mono text-muted-foreground">{doc.chunk_count ?? '—'}</span>
                  </td>
                  <td class="px-4 py-3 text-right">
                    <button on:click={() => deleteDoc(doc.id)}
                            class="p-1.5 rounded-lg text-rose-500 hover:bg-rose-50
                                   dark:hover:bg-rose-900/20 transition-colors">
                      <Trash2 size={15} />
                    </button>
                  </td>
                </tr>
              {/each}
              {#if documents.length === 0}
                <tr><td colspan="4" class="px-4 py-8 text-center text-muted-foreground text-sm">
                  Belum ada dokumen.
                </td></tr>
              {/if}
            </tbody>
          </table>
        </div>
      {/if}

    <!-- ══ LOGS ══ -->
    {:else if activeTab === 'logs'}
      <div class="flex justify-between items-center mb-4">
        <p class="text-sm text-muted-foreground">
          {logsTotal.toLocaleString()} total query ·
          Halaman {logsPage + 1} dari {totalLogPages || 1}
        </p>
        <button on:click={() => loadLogs(logsPage)}
                class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground
                       border rounded-lg px-3 py-2 transition-colors">
          <RefreshCw size={13} /> Refresh
        </button>
      </div>

      {#if logsLoading}
        <div class="flex items-center justify-center py-12 text-muted-foreground gap-2">
          <Loader2 size={18} class="animate-spin" /> Memuat...
        </div>
      {:else if logsError}
        <div class="bg-destructive/10 text-destructive rounded-lg p-4 text-sm">{logsError}</div>
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
                    <p class="line-clamp-2 text-foreground leading-relaxed">{log.query_text}</p>
                    <span class="text-[10px] text-muted-foreground/70 uppercase tracking-wide">
                      {log.detected_language}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-right hidden sm:table-cell">
                    <span class="text-xs font-mono {log.num_results === 0 ? 'text-rose-500' : 'text-emerald-600'}">
                      {log.num_results}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-right hidden sm:table-cell">
                    <span class="text-xs font-mono text-muted-foreground">{log.search_time_ms}ms</span>
                  </td>
                  <td class="px-4 py-3 text-right hidden md:table-cell text-xs text-muted-foreground">
                    {log.created_at?.slice(0,16).replace('T', ' ') ?? '—'}
                  </td>
                </tr>
              {/each}
              {#if logs.length === 0}
                <tr><td colspan="4" class="px-4 py-8 text-center text-muted-foreground text-sm">
                  Belum ada log.
                </td></tr>
              {/if}
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        {#if totalLogPages > 1}
          <div class="flex items-center justify-center gap-2">
            <button disabled={logsPage === 0}
                    on:click={() => loadLogs(logsPage - 1)}
                    class="p-2 rounded-lg border hover:bg-muted disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
              <ChevronLeft size={16} />
            </button>
            <span class="text-sm text-muted-foreground px-2">
              {logsPage + 1} / {totalLogPages}
            </span>
            <button disabled={logsPage >= totalLogPages - 1}
                    on:click={() => loadLogs(logsPage + 1)}
                    class="p-2 rounded-lg border hover:bg-muted disabled:opacity-40 disabled:cursor-not-allowed transition-colors">
              <ChevronRight size={16} />
            </button>
          </div>
        {/if}
      {/if}
    {/if}

  </div>
</div>
