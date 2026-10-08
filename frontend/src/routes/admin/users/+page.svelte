<script lang="ts">
  import { onMount } from 'svelte';
  import { admin } from '$lib/api/client';
  import type { AdminUser } from '$lib/api/client';
  import { Loader2, Trash2, ShieldCheck, ShieldOff, RefreshCw, Search, UserPlus, MailCheck } from 'lucide-svelte';

  let users: AdminUser[] = [];
  let loading = false;
  let error = '';
  let search = '';

  // Add user
  let showAdd = false;
  let addMode: 'single' | 'batch' = 'single';
  let newEmail = ''; let newName = ''; let newRole = 'user';
  let batchCsv = '';
  let addLoading = false; let addError = ''; let addSuccess = '';

  onMount(() => loadUsers());

  async function loadUsers() {
    loading = true; error = '';
    try { users = await admin.users(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  async function addSingle() {
    if (!newEmail.trim()) { addError = 'Email wajib diisi'; return; }
    addLoading = true; addError = ''; addSuccess = '';
    try {
      await admin.createUser(newEmail.trim(), newName.trim() || undefined, newRole);
      addSuccess = `${newEmail} berhasil ditambahkan`;
      newEmail = ''; newName = ''; newRole = 'user';
      users = []; loadUsers();
    } catch (e: unknown) { addError = e instanceof Error ? e.message : String(e); }
    finally { addLoading = false; }
  }

  async function addBatch() {
    const lines = batchCsv.trim().split('\n').filter(l => l.trim() && !l.startsWith('email'));
    if (!lines.length) { addError = 'CSV kosong'; return; }
    addLoading = true; addError = ''; addSuccess = '';
    let ok = 0; let fail = 0;
    for (const line of lines) {
      const [email, name, role] = line.split(',').map(s => s.trim());
      if (!email) { fail++; continue; }
      try { await admin.createUser(email, name || undefined, role || 'user'); ok++; }
      catch { fail++; }
    }
    addSuccess = `Berhasil: ${ok}, Gagal: ${fail}`;
    addLoading = false;
    if (ok > 0) { users = []; loadUsers(); }
  }

  async function toggleUser(u: AdminUser) {
    try {
      await admin.toggleUser(u.id, !u.is_active);
      users = users.map(x => x.id === u.id ? { ...x, is_active: !u.is_active } : x);
    } catch (e) { alert('Gagal: ' + e); }
  }

  async function setRole(u: AdminUser, role: string) {
    try {
      await admin.setRole(u.id, role);
      users = users.map(x => x.id === u.id ? { ...x, role } : x);
    } catch (e) { alert('Gagal: ' + e); }
  }

  async function verifyUser(u: AdminUser) {
    try {
      await admin.verifyUser(u.id);
      users = users.map(x => x.id === u.id ? { ...x, email_verified: true, is_active: true } : x);
    } catch (e) { alert('Gagal: ' + e); }
  }

  async function deleteUser(u: AdminUser) {
    if (!confirm(`Hapus ${u.email}? Tidak dapat dibatalkan.`)) return;
    try { await admin.deleteUser(u.id); users = users.filter(x => x.id !== u.id); }
    catch (e) { alert('Gagal: ' + e); }
  }

  $: filtered = search.trim()
    ? users.filter(u =>
        u.email.toLowerCase().includes(search.toLowerCase()) ||
        (u.display_name ?? '').toLowerCase().includes(search.toLowerCase()))
    : users;
</script>

<svelte:head><title>Pengguna — Admin PMPSTI</title></svelte:head>

<!-- Add User Modal -->
{#if showAdd}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
       on:click|self={() => showAdd = false} role="presentation">
    <div class="bg-card border rounded-2xl shadow-xl w-full max-w-md mx-4 p-6">
      <h3 class="text-base font-semibold mb-4">Tambah Pengguna</h3>
      <div class="flex gap-1 mb-4 p-1 bg-muted rounded-lg">
        <button on:click={() => addMode = 'single'}
          class="flex-1 text-xs py-1.5 rounded-md font-medium transition-colors
                 {addMode === 'single' ? 'bg-background shadow text-foreground' : 'text-muted-foreground'}">
          Satuan
        </button>
        <button on:click={() => addMode = 'batch'}
          class="flex-1 text-xs py-1.5 rounded-md font-medium transition-colors
                 {addMode === 'batch' ? 'bg-background shadow text-foreground' : 'text-muted-foreground'}">
          Batch CSV
        </button>
      </div>
      {#if addMode === 'single'}
        <div class="space-y-3">
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">Email *</label>
            <input bind:value={newEmail} type="email" placeholder="nama@mail.ugm.ac.id"
              class="w-full px-3 py-2 text-sm border rounded-lg bg-background outline-none focus:ring-2 focus:ring-[#0055A5]/40" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">Nama</label>
            <input bind:value={newName} placeholder="Nama lengkap (opsional)"
              class="w-full px-3 py-2 text-sm border rounded-lg bg-background outline-none focus:ring-2 focus:ring-[#0055A5]/40" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">Role</label>
            <select bind:value={newRole} class="w-full px-3 py-2 text-sm border rounded-lg bg-background">
              <option value="user">user</option>
              <option value="admin">admin</option>
            </select>
          </div>
        </div>
      {:else}
        <div class="space-y-2">
          <p class="text-xs text-muted-foreground">Format: <code class="bg-muted px-1 rounded">email,nama,role</code></p>
          <textarea bind:value={batchCsv} rows={5}
            placeholder="budi@mail.ugm.ac.id,Budi,user&#10;siti@mail.ugm.ac.id,Siti,admin"
            class="w-full px-3 py-2 text-xs font-mono border rounded-lg bg-background outline-none focus:ring-2 focus:ring-[#0055A5]/40 resize-none" />
        </div>
      {/if}
      {#if addError}<p class="text-xs text-destructive mt-2">{addError}</p>{/if}
      {#if addSuccess}<p class="text-xs text-emerald-600 mt-2">{addSuccess}</p>{/if}
      <div class="flex gap-2 mt-4 justify-end">
        <button on:click={() => showAdd = false}
          class="px-4 py-2 text-sm border rounded-lg hover:bg-muted transition-colors">Batal</button>
        <button on:click={addMode === 'single' ? addSingle : addBatch} disabled={addLoading}
          class="px-4 py-2 text-sm bg-[#0055A5] text-white rounded-lg hover:bg-[#0044a0]
                 disabled:opacity-50 transition-colors flex items-center gap-2">
          {#if addLoading}<Loader2 size={13} class="animate-spin" />{/if}
          {addMode === 'single' ? 'Tambah' : 'Import'}
        </button>
      </div>
    </div>
  </div>
{/if}

<div class="mb-6 flex items-center justify-between gap-3 flex-wrap">
  <div>
    <h1 class="text-xl font-bold">Pengguna</h1>
    <p class="text-sm text-muted-foreground mt-0.5">{users.length} pengguna terdaftar</p>
  </div>
  <div class="flex items-center gap-2">
    <button on:click={() => { users = []; loadUsers(); }}
      class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground border rounded-lg px-3 py-2 transition-colors">
      <RefreshCw size={13} /> Refresh
    </button>
    <button on:click={() => showAdd = true}
      class="flex items-center gap-1.5 text-sm text-white bg-[#0055A5] hover:bg-[#0044a0] rounded-lg px-3 py-2 transition-colors">
      <UserPlus size={13} /> Tambah
    </button>
  </div>
</div>

<div class="relative mb-4">
  <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
  <input bind:value={search} placeholder="Cari email atau nama..."
    class="w-full pl-8 pr-3 py-2 text-sm border rounded-lg bg-background outline-none focus:ring-2 focus:ring-[#0055A5]/40" />
</div>

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
          <th class="px-4 py-2.5 text-left">Pengguna</th>
          <th class="px-4 py-2.5 text-left hidden sm:table-cell">Role</th>
          <th class="px-4 py-2.5 text-left hidden md:table-cell">Status</th>
          <th class="px-4 py-2.5 text-left hidden md:table-cell">Bergabung</th>
          <th class="px-4 py-2.5 text-right">Aksi</th>
        </tr>
      </thead>
      <tbody class="divide-y">
        {#each filtered as u}
          <tr class="hover:bg-muted/30 transition-colors">
            <td class="px-4 py-3">
              <div class="font-medium">{u.display_name || '—'}</div>
              <div class="text-xs text-muted-foreground">{u.email}</div>
            </td>
            <td class="px-4 py-3 hidden sm:table-cell">
              <select value={u.role}
                on:change={e => setRole(u, e.currentTarget.value)}
                class="text-xs border rounded px-2 py-1 bg-background
                       {u.role === 'admin' ? 'text-[#0055A5] font-semibold' : ''}">
                <option value="user">user</option>
                <option value="admin">admin</option>
              </select>
            </td>
            <td class="px-4 py-3 hidden md:table-cell">
              <div class="flex flex-col gap-1">
                <span class="inline-flex items-center text-xs px-2 py-0.5 rounded-full font-medium w-fit
                             {u.is_active
                               ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400'
                               : 'bg-rose-100 text-rose-600 dark:bg-rose-900/30 dark:text-rose-400'}">
                  {u.is_active ? 'Aktif' : 'Nonaktif'}
                </span>
                {#if !u.email_verified}
                  <span class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium w-fit
                               bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400">
                    ⚠ Belum diverifikasi
                  </span>
                {/if}
              </div>
            </td>
            <td class="px-4 py-3 hidden md:table-cell text-xs text-muted-foreground">
              {u.created_at?.slice(0,10) ?? '—'}
            </td>
            <td class="px-4 py-3 text-right flex justify-end gap-1">
              {#if !u.email_verified}
                <button on:click={() => verifyUser(u)} title="Verifikasi & Aktifkan"
                  class="p-1.5 rounded-lg text-amber-600 hover:text-amber-700 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors">
                  <MailCheck size={15}/>
                </button>
              {/if}
              <button on:click={() => toggleUser(u)} title={u.is_active ? 'Nonaktifkan' : 'Aktifkan'}
                class="p-1.5 rounded-lg transition-colors
                       {u.is_active ? 'text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-900/20'
                                    : 'text-emerald-600 hover:bg-emerald-50 dark:hover:bg-emerald-900/20'}">
                {#if u.is_active}<ShieldOff size={15}/>{:else}<ShieldCheck size={15}/>{/if}
              </button>
              <button on:click={() => deleteUser(u)} title="Hapus"
                class="p-1.5 rounded-lg text-rose-400 hover:text-rose-600 hover:bg-rose-50 dark:hover:bg-rose-900/20 transition-colors">
                <Trash2 size={15}/>
              </button>
            </td>
          </tr>
        {/each}
        {#if filtered.length === 0}
          <tr><td colspan="5" class="px-4 py-8 text-center text-muted-foreground text-sm">Tidak ada pengguna.</td></tr>
        {/if}
      </tbody>
    </table>
  </div>
  <p class="text-xs text-muted-foreground mt-2">{filtered.length} dari {users.length} pengguna</p>
{/if}
