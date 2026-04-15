<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/api/client';
  import { authStore } from '$lib/stores/auth';
  import { MessageSquare, Loader2 } from 'lucide-svelte';

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

<svelte:head><title>Login — PMPSTI RAG</title></svelte:head>

<div class="min-h-screen flex items-center justify-center bg-background p-4">
  <div class="w-full max-w-sm">
    <!-- Logo -->
    <div class="flex items-center justify-center gap-2 mb-8">
      <div class="w-9 h-9 rounded-xl bg-primary flex items-center justify-center">
        <MessageSquare size={18} class="text-primary-foreground" />
      </div>
      <span class="text-xl font-semibold">PMPSTI RAG</span>
    </div>

    <div class="bg-card border rounded-xl p-6 shadow-sm">
      <h1 class="text-lg font-semibold mb-1">Masuk</h1>
      <p class="text-sm text-muted-foreground mb-6">Masukkan akun kamu untuk melanjutkan</p>

      {#if error}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 mb-4">
          {error}
        </div>
      {/if}

      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="text-sm font-medium mb-1.5 block" for="email">Email</label>
          <input
            id="email"
            type="email"
            bind:value={email}
            placeholder="test@test.com"
            class="input"
            required
          />
        </div>
        <div>
          <label class="text-sm font-medium mb-1.5 block" for="password">Password</label>
          <input
            id="password"
            type="password"
            bind:value={password}
            placeholder="••••••••"
            class="input"
            required
          />
        </div>
        <button type="submit" disabled={loading} class="btn-primary w-full">
          {#if loading}
            <Loader2 size={16} class="animate-spin mr-2" />
          {/if}
          Masuk
        </button>
      </form>

      <p class="text-sm text-center text-muted-foreground mt-4">
        Belum punya akun?
        <a href="/register" class="text-foreground font-medium hover:underline">Daftar</a>
      </p>
    </div>
  </div>
</div>