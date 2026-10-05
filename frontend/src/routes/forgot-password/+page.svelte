<script lang="ts">
  import { auth } from '$lib/api/client';

  let email = '';
  let loading = false;
  let success = false;
  let error = '';

  async function submit() {
    if (!email.trim()) { error = 'Email wajib diisi'; return; }
    loading = true; error = '';
    try {
      await auth.forgotPassword(email.trim());
      success = true;
    } catch (e: any) {
      error = e.message || 'Gagal mengirim email reset';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head><title>Lupa Password — PMPSTI</title></svelte:head>

<div class="min-h-screen flex items-center justify-center bg-background px-4">
  <div class="w-full max-w-sm space-y-6">
    <!-- Logo -->
    <div class="text-center">
      <div class="inline-flex items-center justify-center w-12 h-12 rounded-xl bg-[#002147] mb-4">
        <svg viewBox="0 0 24 24" class="w-6 h-6 text-white fill-white" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
        </svg>
      </div>
      <h1 class="text-2xl font-bold text-foreground">Lupa Password</h1>
      <p class="text-sm text-muted-foreground mt-1">
        Masukkan email kamu — kami kirim link reset password
      </p>
    </div>

    {#if success}
      <div class="rounded-xl border bg-emerald-50 dark:bg-emerald-900/20 border-emerald-200 dark:border-emerald-800 p-4 text-center">
        <p class="text-sm font-medium text-emerald-700 dark:text-emerald-300">
          Email terkirim! Cek inbox kamu dan klik link reset password.<br>
          <span class="text-xs font-normal">(Link berlaku 1 jam)</span>
        </p>
      </div>
      <p class="text-center text-sm text-muted-foreground">
        <a href="/login" class="text-[#0055A5] hover:underline font-medium">Kembali ke Login</a>
      </p>
    {:else}
      <form on:submit|preventDefault={submit} class="space-y-4">
        {#if error}
          <div class="rounded-lg border border-rose-200 bg-rose-50 dark:bg-rose-900/20 dark:border-rose-800 px-3 py-2 text-sm text-rose-600 dark:text-rose-400">
            {error}
          </div>
        {/if}

        <div class="space-y-1.5">
          <label for="email" class="text-sm font-medium text-foreground">Email</label>
          <input
            id="email"
            bind:value={email}
            type="email"
            placeholder="nama@mail.ugm.ac.id"
            required
            class="w-full rounded-lg border bg-background px-3 py-2 text-sm
                   focus:outline-none focus:ring-2 focus:ring-[#0055A5]/30 focus:border-[#0055A5]
                   placeholder:text-muted-foreground"
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          class="w-full rounded-lg bg-[#002147] hover:bg-[#003166] text-white
                 py-2.5 text-sm font-medium transition-colors disabled:opacity-60">
          {loading ? 'Mengirim...' : 'Kirim Link Reset'}
        </button>

        <p class="text-center text-sm text-muted-foreground">
          Ingat password? <a href="/login" class="text-[#0055A5] hover:underline font-medium">Login</a>
        </p>
      </form>
    {/if}
  </div>
</div>
