<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/api/client';
  import { authStore } from '$lib/stores/auth';
  import { MessageSquare, Loader2 } from 'lucide-svelte';

  let email = '';
  let password = '';
  let display_name = '';
  let error = '';
  let loading = false;

  async function handleRegister() {
    if (!email || password.length < 8) {
      error = 'Email wajib diisi dan password minimal 8 karakter';
      return;
    }
    loading = true;
    error = '';
    try {
      const res = await auth.register(email, password, display_name || undefined);
      authStore.login(res.token, res.user);
      goto('/chat');
    } catch (e: any) {
      error = e.message ?? 'Pendaftaran gagal';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Daftar — PMPSTI RAG</title></svelte:head>

<div class="min-h-screen flex items-center justify-center bg-background p-4">
  <div class="w-full max-w-sm">
    <div class="flex items-center justify-center gap-2 mb-8">
      <div class="w-9 h-9 rounded-xl bg-primary flex items-center justify-center">
        <MessageSquare size={18} class="text-primary-foreground" />
      </div>
      <span class="text-xl font-semibold">PMPSTI RAG</span>
    </div>

    <div class="bg-card border rounded-xl p-6 shadow-sm">
      <h1 class="text-lg font-semibold mb-1">Buat akun</h1>
      <p class="text-sm text-muted-foreground mb-6">Daftar untuk mulai menggunakan RAG</p>

      {#if error}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 mb-4">
          {error}
        </div>
      {/if}

      <form on:submit|preventDefault={handleRegister} class="space-y-4">
        <div>
          <label class="text-sm font-medium mb-1.5 block" for="name">Nama (opsional)</label>
          <input id="name" type="text" bind:value={display_name} placeholder="Nama kamu" class="input" />
        </div>
        <div>
          <label class="text-sm font-medium mb-1.5 block" for="email">Email</label>
          <input id="email" type="email" bind:value={email} placeholder="kamu@contoh.com" class="input" required />
        </div>
        <div>
          <label class="text-sm font-medium mb-1.5 block" for="password">Password</label>
          <input id="password" type="password" bind:value={password} placeholder="Minimal 8 karakter" class="input" required />
        </div>
        <button type="submit" disabled={loading} class="btn-primary w-full">
          {#if loading}<Loader2 size={16} class="animate-spin mr-2" />{/if}
          Daftar
        </button>
      </form>

      <p class="text-sm text-center text-muted-foreground mt-4">
        Sudah punya akun?
        <a href="/login" class="text-foreground font-medium hover:underline">Masuk</a>
      </p>
    </div>
  </div>
</div>

<style>
  .input { @apply w-full px-3 py-2 text-sm border rounded-lg bg-background focus:outline-none focus:ring-2 focus:ring-ring transition-shadow; }
  .btn-primary { @apply flex items-center justify-center bg-primary text-primary-foreground text-sm font-medium px-4 py-2 rounded-lg hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-opacity; }
</style>
