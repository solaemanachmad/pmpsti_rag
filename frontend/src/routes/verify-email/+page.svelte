<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { CheckCircle2, XCircle, Loader2 } from 'lucide-svelte';

  type State = 'loading' | 'success' | 'error';
  let state: State = 'loading';
  let message = '';

  onMount(async () => {
    const token = $page.url.searchParams.get('token');
    if (!token) { state = 'error'; message = 'Token tidak ditemukan di URL.'; return; }
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

<div class="min-h-screen flex flex-col bg-background">
  <div class="bg-[#002147] h-1.5 w-full"></div>
  <div class="bg-[#0055A5] h-0.5 w-full mb-8"></div>

  <div class="flex-1 flex flex-col items-center justify-center px-4 pb-12">
    <div class="flex flex-col items-center gap-3 mb-8">
      <picture>
          <source srcset="/ugm-logo-white.png" media="(prefers-color-scheme: dark)" />
          <img src="/ugm-logo-blue.png" alt="Logo UGM" class="w-16 h-16" />
        </picture>
      <div class="text-center">
        <div class="font-bold text-lg">PMPSTI RAG</div>
        <div class="text-xs text-muted-foreground">Universitas Gadjah Mada</div>
      </div>
    </div>

    <div class="w-full max-w-sm bg-card border rounded-xl p-6 shadow-sm text-center space-y-3">
      {#if state === 'loading'}
        <Loader2 size={36} class="animate-spin text-[#0055A5] mx-auto" />
        <p class="text-sm text-muted-foreground">Memverifikasi email kamu…</p>

      {:else if state === 'success'}
        <div class="w-14 h-14 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center mx-auto">
          <CheckCircle2 size={26} class="text-green-600 dark:text-green-400" />
        </div>
        <h1 class="text-base font-semibold">Email terverifikasi</h1>
        <p class="text-sm text-muted-foreground">{message}</p>
        <a href="/login" class="btn-primary w-full bg-[#0055A5] hover:bg-[#002147]">
          Masuk sekarang
        </a>

      {:else}
        <div class="w-14 h-14 rounded-full bg-destructive/10 flex items-center justify-center mx-auto">
          <XCircle size={26} class="text-destructive" />
        </div>
        <h1 class="text-base font-semibold">Verifikasi gagal</h1>
        <p class="text-sm text-muted-foreground">{message}</p>
        <a href="/register" class="btn-primary w-full bg-[#0055A5] hover:bg-[#002147]">
          Daftar ulang
        </a>
      {/if}
    </div>
  </div>
</div>
