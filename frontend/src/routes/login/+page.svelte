<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/api/client';
  import { authStore } from '$lib/stores/auth';
  import { Loader2 } from 'lucide-svelte';

  let email = '';
  let password = '';
  let error = '';
  let loading = false;

  async function handleLogin() {
    if (!email || !password) { error = 'Email dan password wajib diisi'; return; }
    loading = true;
    error = '';
    try {
      const res = await auth.login(email, password);
      authStore.login(res.token, res.user);
      goto('/chat');
    } catch (e: any) {
      error = e.message ?? 'Login gagal';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Masuk — PMPSTI RAG</title></svelte:head>

<div class="min-h-screen flex flex-col bg-background">
  <!-- Top bar UGM -->
  <div class="bg-[#002147] h-1.5 w-full"></div>
  <div class="bg-[#0055A5] h-0.5 w-full mb-8"></div>

  <div class="flex-1 flex flex-col items-center justify-center px-4 pb-12">
    <!-- Branding -->
    <div class="flex flex-col items-center gap-3 mb-8">
      <picture>
          <source srcset="/ugm-logo-white.png" media="(prefers-color-scheme: dark)" />
          <img src="/ugm-logo-blue.png" alt="Logo UGM" class="w-16 h-16" />
        </picture>
      <div class="text-center">
        <div class="font-bold text-lg text-foreground">PMPSTI RAG</div>
        <div class="text-xs text-muted-foreground">Asisten Akademik — Universitas Gadjah Mada</div>
      </div>
    </div>

    <!-- Card -->
    <div class="w-full max-w-sm bg-card border rounded-xl p-6 shadow-sm">
      <h1 class="text-base font-semibold mb-1">Masuk ke akun</h1>
      <p class="text-xs text-muted-foreground mb-5">Gunakan email @mail.ugm.ac.id kamu</p>

      {#if error}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive
                    text-xs rounded-lg px-3 py-2.5 mb-4 leading-relaxed">
          {error}
        </div>
      {/if}

      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="text-xs font-medium mb-1.5 block text-muted-foreground" for="email">Email</label>
          <input id="email" type="email" bind:value={email}
            placeholder="nim@mail.ugm.ac.id" class="input" autocomplete="email" required />
        </div>
        <div>
          <label class="text-xs font-medium mb-1.5 block text-muted-foreground" for="password">Password</label>
          <input id="password" type="password" bind:value={password}
            placeholder="••••••••" class="input" autocomplete="current-password" required />
        </div>
        <button type="submit" disabled={loading}
          class="btn-primary w-full mt-1 bg-[#0055A5] hover:bg-[#002147]">
          {#if loading}<Loader2 size={15} class="animate-spin" />{/if}
          Masuk
        </button>
      </form>

      <p class="text-xs text-center text-muted-foreground mt-5">
        Belum punya akun?
        <a href="/register" class="text-[#0055A5] font-semibold hover:underline">Daftar</a>
      </p>
    </div>

    <a href="/" class="mt-5 text-xs text-muted-foreground hover:text-foreground transition-colors">
      ← Kembali ke halaman utama
    </a>
  </div>
</div>
