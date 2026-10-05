<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser, authReady } from '$lib/stores/auth';
  import { admin } from '$lib/api/client';
  import type { QueryLogStats, AdminUser, AdminDocument } from '$lib/api/client';
  import {
    BarChart3, Users, FileText, List,
    Loader2, Trash2, ShieldCheck, ShieldOff,
    RefreshCw, ChevronLeft, ChevronRight,
    Search, UserPlus, Globe, Plus, X, CheckCircle2, AlertCircle
  } from 'lucide-svelte';

  // ── Add User ──
  let showAddUser = false;
  let addMode: 'single' | 'batch' = 'single';
  let newEmail = ''; let newName = ''; let newRole = 'user';
  let batchCsv = '';
  let addLoading = false; let addError = ''; let addSuccess = '';

  async function addSingleUser() {
    if (!newEmail.trim()) { addError = 'Email wajib diisi'; return; }
    addLoading = true; addError = ''; addSuccess = '';
    try {
      await admin.createUser(newEmail.trim(), newName.trim() || undefined, newRole);
      addSuccess = `Pengguna ${newEmail} berhasil ditambahkan`;
      newEmail = ''; newName = ''; newRole = 'user';
      users = []; loadUsers();
    } catch (e: unknown) { addError = e instanceof Error ? e.message : String(e); }
    finally { addLoading = false; }
  }

  async function addBatchUsers() {
    const lines = batchCsv.trim().split('\n').filter(l => l.trim() && !l.startsWith('email'));
    if (!lines.length) { addError = 'CSV kosong'; return; }
    addLoading = true; addError = ''; addSuccess = '';
    let ok = 0; let fail = 0;
    for (const line of lines) {
      const [email, name, role] = line.split(',').map(s => s.trim());
      if (!email) { fail++; continue; }
      try {
        await admin.createUser(email, name || undefined, role || 'user');
        ok++;
      } catch { fail++; }
    }
    addSuccess = `Berhasil: ${ok}, Gagal: ${fail}`;
    addLoading = false;
    if (ok > 0) { users = []; loadUsers(); }
  }

  type Tab = 'stats' | 'users' | 'documents' | 'logs';
  let activeTab: Tab = 'stats';

  // ── Stats ──
  let stats: QueryLogStats | null = null;
  let statsLoading = false;
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

  // ── Ingest URL ──
  let showIngest = false;
  let ingestUrl = '';
  let ingestTitle = '';
  let ingestCategory = '';
  let ingestSubcategory = '';
  let ingestLoading = false;
  let ingestError = '';
  let ingestSuccess = '';

  async function doIngestUrl() {
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
      documents = []; loadDocuments();
    } catch (e: unknown) { ingestError = e instanceof Error ? e.message : String(e); }
    finally { ingestLoading = false; }
  }

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

  // ── Access guard — tunggu authReady agar tidak redirect prematur ──
  onMount(() => {
    const unsubscribe = authReady.subscribe(ready => {
      if (!ready) return;  // belum selesai verify
      unsubscribe();
      const u = getCurrentUser();
      if (!u) { goto('/login'); return; }
      if (u.role !== 'admin') { goto('/chat'); return; }
      loadStats();
    });
  });

  // Helper: baca nilai currentUser store saat ini (satu kali)
  function getCurrentUser() {
    let val: import('$lib/api/client').UserPublic | null = null;
    const unsub = currentUser.subscribe(u => { val = u; });
    unsub();
    return val;
  }

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
    if (tab === 'stats' && !stats && !statsLoading) loadStats();
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

  async function deleteUser(u: AdminUser) {
    if (!confirm(`Hapus user ${u.email}? Tindakan ini tidak dapat dibatalkan.`)) return;
    try {
      await admin.deleteUser(u.id);
      users = users.filter(x => x.id !== u.id);
    } catch (e) { alert('Gagal menghapus: ' + e); }
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

<div class="h-full overflow-y-auto">
  <div class="max-w-5xl mx-auto px-4 py-6">
    <!-- Page title -->
    <div class="mb-6">
      <h1 class="text-xl font-bold text-foreground">Panel Admin</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Kelola pengguna, dokumen, dan pantau aktivitas sistem</p>
    </div>

    <!-- Tab navigation -->
    <nav class="flex gap-1 mb-6 border-b">
      {#each ([
        { id: 'stats',     icon: BarChart3,  label: 'Dashboard' },
        { id: 'users',     icon: Users,      label: 'Pengguna' },
        { id: 'documents', icon: FileText,   label: 'Dokumen' },
        { id: 'logs',      icon: List,       label: 'Log Query' },
      ]) as item}
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
      <!-- Add User Modal -->
      {#if showAddUser}
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
             on:click|self={() => showAddUser = false} role="presentation">
          <div class="bg-card border rounded-2xl shadow-xl w-full max-w-md mx-4 p-6">
            <h3 class="text-base font-semibold mb-4">Tambah Pengguna</h3>

            <!-- Toggle satuan/batch -->
            <div class="flex gap-1 mb-4 p-1 bg-muted rounded-lg">
              <button on:click={() => addMode = 'single'}
                class="flex-1 text-xs py-1.5 rounded-md font-medium transition-colors
                       {addMode === 'single' ? 'bg-background shadow text-foreground' : 'text-muted-foreground'}">
                Satuan
              </button>
              <button on:click={() => addMode = 'batch'}
                class="flex-1 text-xs py-1.5 rounded-md font-medium transition-colors
                       {addMode === 'batch' ? 'bg-background shadow text-foreground' : 'text-muted-foreground'}">
                Batch (CSV)
              </button>
            </div>

            {#if addMode === 'single'}
              <div class="space-y-3">
                <div>
                  <label class="text-xs text-muted-foreground mb-1 block">Email *</label>
                  <input bind:value={newEmail} type="email" placeholder="mahasiswa@mail.ugm.ac.id"
                    class="w-full px-3 py-2 text-sm border rounded-lg bg-background outline-none
                           focus:ring-2 focus:ring-[#0055A5]/40" />
                </div>
                <div>
                  <label class="text-xs text-muted-foreground mb-1 block">Nama</label>
                  <input bind:value={newName} placeholder="Nama lengkap (opsional)"
                    class="w-full px-3 py-2 text-sm border rounded-lg bg-background outline-none
                           focus:ring-2 focus:ring-[#0055A5]/40" />
                </div>
                <div>
                  <label class="text-xs text-muted-foreground mb-1 block">Role</label>
                  <select bind:value={newRole}
                    class="w-full px-3 py-2 text-sm border rounded-lg bg-background outline-none">
                    <option value="user">user</option>
                    <option value="admin">admin</option>
                  </select>
                </div>
                {#if addError}<p class="text-xs text-destructive">{addError}</p>{/if}
                {#if addSuccess}<p class="text-xs text-emerald-600">{addSuccess}</p>{/if}
              </div>
            {:else}
              <div class="space-y-3">
                <p class="text-xs text-muted-foreground">
                  Format CSV: <code class="bg-muted px-1 rounded">email,nama,role</code><br>
                  Contoh:<br>
                  <code class="bg-muted px-1 rounded text-[11px]">
                    budi@mail.ugm.ac.id,Budi Santoso,user
                  </code>
                </p>
                <textarea bind:value={batchCsv} rows={6}
                  placeholder="email,nama,role&#10;budi@mail.ugm.ac.id,Budi,user&#10;siti@mail.ugm.ac.id,Siti,user"
                  class="w-full px-3 py-2 text-xs font-mono border rounded-lg bg-background outline-none
                         focus:ring-2 focus:ring-[#0055A5]/40 resize-none" />
                {#if addError}<p class="text-xs text-destructive">{addError}</p>{/if}
                {#if addSuccess}<p class="text-xs text-emerald-600">{addSuccess}</p>{/if}
              </div>
            {/if}

            <div class="flex gap-2 mt-4 justify-end">
              <button on:click={() => showAddUser = false}
                class="px-4 py-2 text-sm border rounded-lg hover:bg-muted transition-colors">
                Batal
              </button>
              <button on:click={addMode === 'single' ? addSingleUser : addBatchUsers}
                      disabled={addLoading}
                class="px-4 py-2 text-sm bg-[#0055A5] text-white rounded-lg
                       hover:bg-[#0044a0] disabled:opacity-50 transition-colors flex items-center gap-2">
                {#if addLoading}<Loader2 size={13} class="animate-spin" />{/if}
                {addMode === 'single' ? 'Tambah' : 'Import'}
              </button>
            </div>
          </div>
        </div>
      {/if}

      <div class="flex items-center gap-3 mb-4 flex-wrap">
        <div class="relative flex-1 min-w-[180px]">
          <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input bind:value={userSearch} placeholder="Cari email / nama..."
                 class="w-full pl-8 pr-3 py-2 text-sm border rounded-lg bg-background outline-none
                        focus:ring-2 focus:ring-[#0055A5]/40 focus:border-[#0055A5]/40" />
        </div>
        <button on:click={() => showAddUser = true}
                class="flex items-center gap-1.5 text-sm text-white bg-[#0055A5]
                       hover:bg-[#0044a0] rounded-lg px-3 py-2 transition-colors">
          <UserPlus size={13} /> Tambah
        </button>
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
                            on:change={e => setRole(u, /** @type {HTMLSelectElement} */ (e.target).value)}
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
                    <button on:click={() => deleteUser(u)}
                            title="Hapus user"
                            class="p-1.5 rounded-lg text-rose-400 hover:text-rose-600
                                   hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors ml-1">
                      <Trash2 size={15}/>
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
        <div class="flex items-center gap-2">
          <button on:click={() => { documents = []; loadDocuments(); }}
                  class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground
                         border rounded-lg px-3 py-2 transition-colors">
            <RefreshCw size={13} /> Refresh
          </button>
          <button on:click={() => { showIngest = !showIngest; ingestError = ''; ingestSuccess = ''; }}
                  class="flex items-center gap-1.5 text-sm text-white bg-[#002147]
                         hover:bg-[#003580] px-3 py-2 rounded-lg transition-colors">
            <Globe size={13} /> Tambah URL
          </button>
        </div>
      </div>

      {#if ingestSuccess}
        <div class="flex items-center gap-2 bg-emerald-50 dark:bg-emerald-900/20 text-emerald-700
                    dark:text-emerald-400 border border-emerald-200 dark:border-emerald-800
                    rounded-lg px-4 py-2.5 text-sm mb-4">
          <CheckCircle2 size={15} />{ingestSuccess}
        </div>
      {/if}

      {#if showIngest}
        <div class="bg-card border rounded-xl p-5 mb-4">
          <div class="flex items-center justify-between mb-4">
            <h3 class="font-semibold text-sm">Tambah Sumber dari URL</h3>
            <button on:click={() => showIngest = false} class="text-muted-foreground hover:text-foreground">
              <X size={15} />
            </button>
          </div>
          <div class="grid gap-3">
            <div>
              <label class="block text-xs font-medium text-muted-foreground mb-1">URL <span class="text-rose-500">*</span></label>
              <input bind:value={ingestUrl} placeholder="https://example.com/halaman"
                     class="w-full text-sm border rounded-lg px-3 py-2 bg-background focus:ring-1 focus:ring-[#0055A5] outline-none" />
            </div>
            <div class="grid sm:grid-cols-3 gap-3">
              <div>
                <label class="block text-xs font-medium text-muted-foreground mb-1">Judul (opsional)</label>
                <input bind:value={ingestTitle} placeholder="Judul dokumen"
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
              <div class="flex items-center gap-2 text-rose-600 text-sm">
                <AlertCircle size={14} />{ingestError}
              </div>
            {/if}
            <div class="flex justify-end">
              <button on:click={doIngestUrl} disabled={ingestLoading}
                      class="flex items-center gap-2 text-sm text-white bg-[#0055A5] hover:bg-[#003580]
                             disabled:opacity-50 px-4 py-2 rounded-lg transition-colors">
                {#if ingestLoading}<Loader2 size={14} class="animate-spin" />Memproses...{:else}<Plus size={14} />Proses & Indeks{/if}
              </button>
            </div>
          </div>
        </div>
      {/if}

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
