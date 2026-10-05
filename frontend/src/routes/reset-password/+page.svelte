<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { auth } from '$lib/api/client';

  let token = '';
  let password = '';
  let confirm = '';
  let loading = false;
  let success = false;
  let error = '';

  onMount(() => {
    token = $page.url.searchParams.get('token') ?? '';
    if (!token) { error = 'Token tidak ditemukan. Minta link reset ulang.'; }
  });

  async function submit() {
    if (password.length < 8) { error = 'Password minimal 8 karakter'; return; }
    if (password !== confirm) { error = 'Konfirmasi password tidak cocok'; return; }
    loading = true; error = '';
    try {
      await auth.resetPassword(token, password);
      success = true;
      setTimeout(() => goto('/login'), 2500);
    } catch (e: any) {
      error = e.message || 'Gagal reset password';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Reset Password — PMPSTI</title></svelte:head>

<div class="min-h-screen flex items-center justify-center bg-background px-4">
  <div class="w-full max-w-sm space-y-6">
    <!-- Logo -->
    <div class="text-center">
      <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-[#002147] mb-4">
        <svg viewBox="0 0 24 24" class="w-6 h-6 fill-none stroke-white stroke-2" xmlns="http://www.w3.org/2000/svg">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
          <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-foreground">Buat Password Baru</h1>
      <p class="text-sm text-muted-foreground mt-1">Minimal 8 karakter</p>
    </div>

    {#if success}
      <div class="rounded-xl border bg-emerald-50 dark:bg-emerald-900/20 border-emerald-200 dark:border-emerald-800 p-4 text-center">
        <p class="text-sm font-medium text-emerald-700 dark:text-emerald-300">
          Password berhasil diubah!<br>
          <span class="text-xs font-normal">Mengalihkan ke halaman login...</span>
        </p>
      </div>
    {:else}
      <form on:submit|preventDefault={submit} class="space-y-4">
        {#if error}
          <div class="rounded-lg border border-rose-200 bg-rose-50 dark:bg-rose-900/20 dark:border-rose-800 px-3 py-2 text-sm text-rose-600 dark:text-rose-400">
            {error}
          </div>
        {/if}

        <div class="space-y-1.5">
          <label for="pw" class="text-sm font-medium text-foreground">Password Baru</label>
          <input
            id="pw"
            bind:value={password}
            type="password"
            placeholder="Minimal 8 karakter"
            required
            class="w-full rounded-lg border bg-background px-3 py-2 text-sm
                   focus:outline-none focus:ring-2 focus:ring-[#0055A5]/30 focus:border-[#0055A5]
                   placeholder:text-muted-foreground"
          />
        </div>

        <div class="space-y-1.5">
          <label for="confirm" class="text-sm font-medium text-foreground">Konfirmasi Password</label>
          <input
            id="confirm"
            bind:value={confirm}
            type="password"
            placeholder="Ulangi password"
            required
            class="w-full rounded-lg border bg-background px-3 py-2 text-sm
                   focus:outline-none focus:ring-2 focus:ring-[#0055A5]/30 focus:border-[#0055A5]
                   placeholder:text-muted-foreground"
          />
        </div>

        <button
          type="submit"
          disabled={loading || !token}
          class="w-full rounded-lg bg-[#002147] hover:bg-[#003166] text-white
                 py-2.5 text-sm font-medium transition-colors disabled:opacity-60">
          {loading ? 'Menyimpan...' : 'Simpan Password'}
        </button>

        <p class="text-center text-sm text-muted-foreground">
          <a href="/forgot-password" class="text-[#0055A5] hover:underline font-medium">Minta link baru</a>
        </p>
      </form>
    {/if}
  </div>
</div>
