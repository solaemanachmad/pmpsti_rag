<script lang="ts">
  import { onMount } from 'svelte';
  import {
    admin
  } from '$lib/api/client';
  import type {
    QueryLogStats,
    AdminUser,
    AdminSession,
    AdminDocument
  } from '$lib/api/client';
  import {
    BarChart3, Users, MessageSquare, FileText,
    Search, Clock, AlertCircle, Loader2,
    Trash2, Shield, ShieldOff,
    RefreshCw, TrendingUp
  } from 'lucide-svelte';

  // ── Tab state ──
  type Tab = 'stats' | 'users' | 'sessions' | 'documents';
  let activeTab: Tab = 'stats';

  // ── Stats ──
  let stats: QueryLogStats | null = null;
  let statsLoading = true;
  let statsError = '';

  // ── Users ──
  let users: AdminUser[] = [];
  let usersLoading = false;
  let usersError = '';

  // ── Sessions ──
  let sessions: AdminSession[] = [];
  let sessionsLoading = false;
  let sessionsError = '';

  // ── Documents ──
  let documents: AdminDocument[] = [];
  let docsLoading = false;
  let docsError = '';

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    statsLoading = true; statsError = '';
    try { stats = await admin.stats(); }
    catch (e: unknown) { statsError = e instanceof Error ? e.message : String(e); }
    finally { statsLoading = false; }
  }

  async function loadUsers() {
    usersLoading = true; usersError = '';
    try { users = await admin.listUsers(); }
    catch (e: unknown) { usersError = e instanceof Error ? e.message : String(e); }
    finally { usersLoading = false; }
  }

  async function loadSessions() {
    sessionsLoading = true; sessionsError = '';
    try { sessions = await admin.listSessions(); }
    catch (e: unknown) { sessionsError = e instanceof Error ? e.message : String(e); }
    finally { sessionsLoading = false; }
  }

  async function loadDocuments() {
    docsLoading = true; docsError = '';
    try { documents = await admin.listDocuments(); }
    catch (e: unknown) { docsError = e instanceof Error ? e.message : String(e); }
    finally { docsLoading = false; }
  }

  async function setRole(u: AdminUser, role: string) {
    try {
      await admin.setRole(u.id, role);
      users = users.map(x => x.id === u.id ? { ...x, role } : x);
    } catch (e: unknown) {
      alert('Gagal mengubah role: ' + (e instanceof Error ? e.message : String(e)));
    }
  }

  async function toggleActive(u: AdminUser) {
    try {
      await admin.toggleActive(u.id, !u.is_active);
      users = users.map(x => x.id === u.id ? { ...x, is_active: !x.is_active } : x);
    } catch (e: unknown) {
      alert('Gagal mengubah status: ' + (e instanceof Error ? e.message : String(e)));
    }
  }

  async function deleteSession(s: AdminSession) {
    if (!confirm(`Hapus session "${s.title || 'Tanpa judul'}"?`)) return;
    try {
      await admin.deleteSession(s.id);
      sessions = sessions.filter(x => x.id !== s.id);
    } catch (e: unknown) {
      alert('Gagal menghapus session: ' + (e instanceof Error ? e.message : String(e)));
    }
  }

  async function deleteDocument(doc: AdminDocument) {
    if (!confirm(`Hapus dokumen "${doc.document_id}" dan semua ${doc.chunk_count} chunk-nya?`)) return;
    try {
      await admin.deleteDocument(doc.document_id);
      documents = documents.filter(d => d.document_id !== doc.document_id);
    } catch (e: unknown) {
      alert('Gagal menghapus dokumen: ' + (e instanceof Error ? e.message : String(e)));
    }
  }

  function switchTab(tab: Tab) {
    activeTab = tab;
    if (tab === 'users' && users.length === 0) loadUsers();
    if (tab === 'sessions' && sessions.length === 0) loadSessions();
    if (tab === 'documents' && documents.length === 0) loadDocuments();
  }

  function pct(val: number, total: number): number {
    if (!total) return 0;
    return Math.round((val / total) * 100);
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleDateString('id-ID', {
      day: 'numeric', month: 'short', year: 'numeric'
    });
  }

  // Tab definitions (computed outside template to avoid TypeScript issues)
  const tabs: Array<{ id: Tab; label: string; icon: typeof BarChart3 }> = [
    { id: 'stats',     label: 'Statistik',  icon: BarChart3 },
    { id: 'users',     label: 'Users',      icon: Users },
    { id: 'sessions',  label: 'Sessions',   icon: MessageSquare },
    { id: 'documents', label: 'Dokumen',    icon: FileText }
  ];
</script>

<svelte:head><title>Admin — PMPSTI RAG</title></svelte:head>

<div class="h-full overflow-y-auto">
  <!-- Header -->
  <div class="border-b bg-card px-6 py-4">
    <h1 class="text-lg font-semibold">Admin Panel</h1>
    <p class="text-sm text-muted-foreground mt-0.5">Kelola sistem RAG</p>
  </div>

  <!-- Tabs -->
  <div class="border-b bg-card px-6">
    <div class="flex gap-0 -mb-px">
      {#each tabs as tab}
        <button
          on:click={() => switchTab(tab.id)}
          class="flex items-center gap-2 px-4 py-3 text-sm border-b-2 transition-colors {activeTab === tab.id
            ? 'border-primary text-primary font-medium'
            : 'border-transparent text-muted-foreground hover:text-foreground'}"
        >
          <svelte:component this={tab.icon} size={14} />
          {tab.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="p-6">

    <!-- ══════════ STATS ══════════ -->
    {#if activeTab === 'stats'}

      {#if statsLoading}
        <div class="flex items-center gap-2 text-muted-foreground py-12 justify-center">
          <Loader2 size={18} class="animate-spin" /><span class="text-sm">Memuat statistik...</span>
        </div>

      {:else if statsError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-4 py-3 flex items-center gap-2">
          <AlertCircle size={14} />{statsError}
        </div>

      {:else if stats}
        <!-- Stat cards -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
          <div class="bg-card border rounded-xl p-4">
            <div class="flex items-center gap-2 text-muted-foreground mb-2">
              <Search size={14} />
              <span class="text-xs font-medium uppercase tracking-wide">Total Query</span>
            </div>
            <p class="text-xl font-semibold">{stats.total_queries.toLocaleString()}</p>
          </div>
          <div class="bg-card border rounded-xl p-4">
            <div class="flex items-center gap-2 text-muted-foreground mb-2">
              <TrendingUp size={14} />
              <span class="text-xs font-medium uppercase tracking-wide">Query Unik</span>
            </div>
            <p class="text-xl font-semibold">{stats.unique_queries.toLocaleString()}</p>
          </div>
          <div class="bg-card border rounded-xl p-4">
            <div class="flex items-center gap-2 text-muted-foreground mb-2">
              <Clock size={14} />
              <span class="text-xs font-medium uppercase tracking-wide">Rata-rata Waktu</span>
            </div>
            <p class="text-xl font-semibold">{Math.round(stats.avg_search_time_ms)} ms</p>
          </div>
          <div class="bg-card border rounded-xl p-4">
            <div class="flex items-center gap-2 text-muted-foreground mb-2">
              <AlertCircle size={14} />
              <span class="text-xs font-medium uppercase tracking-wide">Nol Hasil</span>
            </div>
            <p class="text-xl font-semibold">
              {stats.zero_result_queries}
              <span class="text-sm font-normal text-muted-foreground">
                ({pct(stats.zero_result_queries, stats.total_queries)}%)
              </span>
            </p>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-4">
          <!-- Queries per day -->
          <div class="bg-card border rounded-xl p-4">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-sm font-medium">Query per hari</h2>
              <button on:click={loadStats} class="text-muted-foreground hover:text-foreground p-1 rounded transition-colors">
                <RefreshCw size={13} />
              </button>
            </div>
            {#if stats.queries_per_day.length === 0}
              <p class="text-sm text-muted-foreground text-center py-4">Belum ada data</p>
            {:else}
              {@const maxVal = Math.max(...stats.queries_per_day.map(([, n]) => n), 1)}
              <div class="space-y-1.5">
                {#each stats.queries_per_day.slice(0, 10) as [day, count]}
                  <div class="flex items-center gap-2 text-xs">
                    <span class="text-muted-foreground w-20 shrink-0">{day}</span>
                    <div class="flex-1 bg-muted rounded-full h-1.5 overflow-hidden">
                      <div class="h-full bg-primary rounded-full transition-all" style="width:{pct(count, maxVal)}%"></div>
                    </div>
                    <span class="w-6 text-right font-medium">{count}</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Top queries -->
          <div class="bg-card border rounded-xl p-4">
            <h2 class="text-sm font-medium mb-4">Query terpopuler</h2>
            {#if stats.top_queries.length === 0}
              <p class="text-sm text-muted-foreground text-center py-4">Belum ada data</p>
            {:else}
              <div class="space-y-2">
                {#each stats.top_queries.slice(0, 8) as [q, count], i}
                  <div class="flex items-center gap-2 text-xs">
                    <span class="w-5 text-center text-muted-foreground shrink-0">{i + 1}</span>
                    <span class="flex-1 truncate" title={q}>{q}</span>
                    <span class="bg-muted px-1.5 py-0.5 rounded font-medium shrink-0">{count}×</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>

        <!-- Language + Domain distribution -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="bg-card border rounded-xl p-4">
            <h2 class="text-sm font-medium mb-4">Distribusi bahasa</h2>
            {#if stats.language_distribution.length === 0}
              <p class="text-sm text-muted-foreground text-center py-4">Belum ada data</p>
            {:else}
              {@const total = stats.language_distribution.reduce((a, [, n]) => a + n, 0)}
              <div class="space-y-2">
                {#each stats.language_distribution as [label, count]}
                  <div class="flex items-center gap-2 text-xs">
                    <span class="w-24 text-muted-foreground truncate capitalize shrink-0">{label || 'unknown'}</span>
                    <div class="flex-1 bg-muted rounded-full h-1.5 overflow-hidden">
                      <div class="h-full bg-primary/70 rounded-full transition-all" style="width:{pct(count, total)}%"></div>
                    </div>
                    <span class="w-10 text-right text-muted-foreground">{pct(count, total)}%</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
          <div class="bg-card border rounded-xl p-4">
            <h2 class="text-sm font-medium mb-4">Distribusi domain</h2>
            {#if stats.domain_distribution.length === 0}
              <p class="text-sm text-muted-foreground text-center py-4">Belum ada data</p>
            {:else}
              {@const total = stats.domain_distribution.reduce((a, [, n]) => a + n, 0)}
              <div class="space-y-2">
                {#each stats.domain_distribution as [label, count]}
                  <div class="flex items-center gap-2 text-xs">
                    <span class="w-24 text-muted-foreground truncate capitalize shrink-0">{label || 'unknown'}</span>
                    <div class="flex-1 bg-muted rounded-full h-1.5 overflow-hidden">
                      <div class="h-full bg-primary/70 rounded-full transition-all" style="width:{pct(count, total)}%"></div>
                    </div>
                    <span class="w-10 text-right text-muted-foreground">{pct(count, total)}%</span>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        </div>
      {/if}

    <!-- ══════════ USERS ══════════ -->
    {:else if activeTab === 'users'}
      <div class="flex items-center justify-between mb-4">
        <p class="text-sm text-muted-foreground">{users.length} user terdaftar</p>
        <button
          on:click={loadUsers}
          class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground border rounded-lg px-3 py-1.5 transition-colors"
        >
          <RefreshCw size={12} />Refresh
        </button>
      </div>

      {#if usersLoading}
        <div class="flex items-center gap-2 text-muted-foreground py-12 justify-center">
          <Loader2 size={18} class="animate-spin" /><span class="text-sm">Memuat...</span>
        </div>
      {:else if usersError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-4 py-3">{usersError}</div>
      {:else}
        <div class="border rounded-xl overflow-hidden bg-card">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-muted/40">
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Email</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Nama</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Role</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Status</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Bergabung</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y">
              {#each users as u (u.id)}
                <tr class="hover:bg-muted/20 transition-colors">
                  <td class="px-4 py-3 font-medium text-sm">{u.email}</td>
                  <td class="px-4 py-3 text-muted-foreground hidden md:table-cell">{u.display_name || '—'}</td>
                  <td class="px-4 py-3">
                    <select
                      value={u.role}
                      on:change={(e) => setRole(u, e.currentTarget.value)}
                      class="text-xs border rounded px-2 py-1 bg-background focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="user">user</option>
                      <option value="admin">admin</option>
                    </select>
                  </td>
                  <td class="px-4 py-3">
                    <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {u.is_active
                      ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300'
                      : 'bg-muted text-muted-foreground'}">
                      {u.is_active ? 'Aktif' : 'Nonaktif'}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{fmtDate(u.created_at)}</td>
                  <td class="px-4 py-3 text-right">
                    <button
                      on:click={() => toggleActive(u)}
                      class="p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
                      title={u.is_active ? 'Nonaktifkan' : 'Aktifkan'}
                    >
                      {#if u.is_active}
                        <ShieldOff size={14} />
                      {:else}
                        <Shield size={14} />
                      {/if}
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if users.length === 0 && !usersLoading}
            <div class="text-center py-12 text-sm text-muted-foreground">Belum ada user</div>
          {/if}
        </div>
      {/if}

    <!-- ══════════ SESSIONS ══════════ -->
    {:else if activeTab === 'sessions'}
      <div class="flex items-center justify-between mb-4">
        <p class="text-sm text-muted-foreground">{sessions.length} session</p>
        <button
          on:click={loadSessions}
          class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground border rounded-lg px-3 py-1.5 transition-colors"
        >
          <RefreshCw size={12} />Refresh
        </button>
      </div>

      {#if sessionsLoading}
        <div class="flex items-center gap-2 text-muted-foreground py-12 justify-center">
          <Loader2 size={18} class="animate-spin" /><span class="text-sm">Memuat...</span>
        </div>
      {:else if sessionsError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-4 py-3">{sessionsError}</div>
      {:else}
        <div class="border rounded-xl overflow-hidden bg-card">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-muted/40">
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Judul</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">User</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Pesan</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Terakhir aktif</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y">
              {#each sessions as s (s.id)}
                <tr class="hover:bg-muted/20 transition-colors">
                  <td class="px-4 py-3 font-medium max-w-[200px] truncate">{s.title || 'Tanpa judul'}</td>
                  <td class="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{s.user_email}</td>
                  <td class="px-4 py-3 text-center">
                    <span class="bg-muted px-2 py-0.5 rounded text-xs">{s.message_count}</span>
                  </td>
                  <td class="px-4 py-3 text-muted-foreground text-xs hidden md:table-cell">{fmtDate(s.updated_at)}</td>
                  <td class="px-4 py-3 text-right">
                    <button
                      on:click={() => deleteSession(s)}
                      class="p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                    >
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if sessions.length === 0 && !sessionsLoading}
            <div class="text-center py-12 text-sm text-muted-foreground">Belum ada session</div>
          {/if}
        </div>
      {/if}

    <!-- ══════════ DOCUMENTS ══════════ -->
    {:else if activeTab === 'documents'}
      <div class="flex items-center justify-between mb-4">
        <p class="text-sm text-muted-foreground">
          {documents.length} dokumen ·
          {documents.reduce((a, d) => a + d.chunk_count, 0).toLocaleString()} chunks
        </p>
        <button
          on:click={loadDocuments}
          class="flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground border rounded-lg px-3 py-1.5 transition-colors"
        >
          <RefreshCw size={12} />Refresh
        </button>
      </div>

      {#if docsLoading}
        <div class="flex items-center gap-2 text-muted-foreground py-12 justify-center">
          <Loader2 size={18} class="animate-spin" /><span class="text-sm">Memuat...</span>
        </div>
      {:else if docsError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-4 py-3">{docsError}</div>
      {:else}
        <div class="border rounded-xl overflow-hidden bg-card">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b bg-muted/40">
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Dokumen</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden md:table-cell">Kategori</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground hidden sm:table-cell">Tipe</th>
                <th class="text-left px-4 py-3 text-xs font-medium text-muted-foreground">Chunks</th>
                <th class="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody class="divide-y">
              {#each documents as doc (doc.document_id)}
                <tr class="hover:bg-muted/20 transition-colors">
                  <td class="px-4 py-3">
                    <div class="font-medium truncate max-w-[200px]" title={doc.title || doc.document_id}>
                      {doc.title || doc.document_id}
                    </div>
                    <div class="text-xs text-muted-foreground truncate max-w-[200px]" title={doc.source_url}>
                      {doc.source_url || doc.document_id}
                    </div>
                  </td>
                  <td class="px-4 py-3 hidden md:table-cell">
                    <span class="bg-muted px-2 py-0.5 rounded text-xs">{doc.category || '—'}</span>
                  </td>
                  <td class="px-4 py-3 text-muted-foreground text-xs hidden sm:table-cell uppercase">
                    {doc.document_type || '—'}
                  </td>
                  <td class="px-4 py-3">
                    <span class="bg-muted px-2 py-0.5 rounded text-xs font-mono">{doc.chunk_count}</span>
                  </td>
                  <td class="px-4 py-3 text-right">
                    <button
                      on:click={() => deleteDocument(doc)}
                      class="p-1.5 rounded-md text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                    >
                      <Trash2 size={14} />
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          {#if documents.length === 0 && !docsLoading}
            <div class="text-center py-12 text-sm text-muted-foreground">Belum ada dokumen</div>
          {/if}
        </div>
      {/if}

    {/if}
  </div>
</div>