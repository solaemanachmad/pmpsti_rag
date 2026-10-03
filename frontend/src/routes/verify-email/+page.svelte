<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { MessageSquare, CheckCircle2, XCircle, Loader2 } from 'lucide-svelte';

  type State = 'loading' | 'success' | 'error';
  let state: State = 'loading';
  let message = '';

  onMount(async () => {
    const token = $page.url.searchParams.get('token');
    if (!token) {
      state = 'error';
      message = 'Token tidak ditemukan di URL.';
      return;
    }

    try {
      const res = await fetch(`/api/auth/verify/${encodeURIComponent(token)}`);
      const json = await res.json();
      if (res.ok) {
        state = 'success';
        message = json.data?.message ?? 'Email berhasil diverifikasi!';
      } else {
        state = 'error';
        message = json.error?.message ?? 'Token tidak valid atau sudah kedaluwarsa.';
      }
    } catch {
      state = 'error';
      message = 'Gagal menghubungi server. Coba lagi nanti.';
    }
  });
</script>

<svelte:head><title>Verifikasi Email — PMPSTI RAG</title></svelte:head>

<div class="min-h-screen flex items-center justify-center bg-background p-4">
  <div class="w-full max-w-sm">
    <div class="flex items-center justify-center gap-2 mb-8">
      <div class="w-9 h-9 rounded-xl bg-primary flex items-center justify-center">
        <MessageSquare size={18} class="text-primary-foreground" />
      </div>
      <span class="text-xl font-semibold">PMPSTI RAG</span>
    </div>

    <div class="bg-card border rounded-xl p-6 shadow-sm text-center space-y-4">
      {#if state === 'loading'}
        <Loader2 size={40} class="animate-spin text-primary mx-auto" />
        <p class="text-sm text-muted-foreground">Memverifikasi email kamu…</p>

      {:else if state === 'success'}
        <div class="flex items-center justify-center w-14 h-14 rounded-full bg-green-100 dark:bg-green-900/30 mx-auto">
          <CheckCircle2 size={28} class="text-green-600 dark:text-green-400" />
        </div>
        <h1 class="text-lg font-semibold">Email Terverifikasi!</h1>
        <p class="text-sm text-muted-foreground">{message}</p>
        <a href="/login" class="btn-primary w-full inline-block text-center">
          Login Sekarang
        </a>

      {:else}
        <div class="flex items-center justify-center w-14 h-14 rounded-full bg-destructive/10 mx-auto">
          <XCircle size={28} class="text-destructive" />
        </div>
        <h1 class="text-lg font-semibold">Verifikasi Gagal</h1>
        <p class="text-sm text-muted-foreground">{message}</p>
        <a href="/register" class="btn-primary w-full inline-block text-center">
          Daftar Ulang
        </a>
      {/if}
    </div>
  </div>
</div>
