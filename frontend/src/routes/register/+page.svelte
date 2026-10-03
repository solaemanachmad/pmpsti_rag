<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth } from '$lib/api/client';
  import { MessageSquare, Loader2, MailCheck } from 'lucide-svelte';

  let email = '';
  let password = '';
  let display_name = '';
  let error = '';
  let loading = false;
  let registered = false;

  async function handleRegister() {
    if (!email || password.length < 8) {
      error = 'Email wajib diisi dan password minimal 8 karakter';
      return;
    }
    if (!email.toLowerCase().endsWith('@mail.ugm.ac.id')) {
      error = 'Registrasi hanya untuk email @mail.ugm.ac.id';
      return;
    }
    loading = true;
    error = '';
    try {
      await auth.register(email, password, display_name || undefined);
      registered = true;
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

    {#if registered}
      <!-- ── Sukses: instruksikan cek email ── -->
      <div class="bg-card border rounded-xl p-6 shadow-sm text-center space-y-4">
        <div class="flex items-center justify-center w-14 h-14 rounded-full bg-primary/10 mx-auto">
          <MailCheck size={28} class="text-primary" />
        </div>
        <h1 class="text-lg font-semibold">Cek email kamu!</h1>
        <p class="text-sm text-muted-foreground">
          Kami sudah mengirim link verifikasi ke<br>
          <span class="font-medium text-foreground">{email}</span>
        </p>
        <p class="text-sm text-muted-foreground">
          Klik link tersebut untuk mengaktifkan akun, lalu login.
          Link berlaku selama <strong>24 jam</strong>.
        </p>
        <a href="/login" class="btn-primary w-full inline-block text-center">
          Pergi ke halaman Login
        </a>
      </div>
    {:else}
      <div class="bg-card border rounded-xl p-6 shadow-sm">
        <h1 class="text-lg font-semibold mb-1">Buat akun</h1>
        <p class="text-sm text-muted-foreground mb-6">
          Khusus email <strong>@mail.ugm.ac.id</strong>
        </p>

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
            <label class="text-sm font-medium mb-1.5 block" for="email">Email UGM</label>
            <input id="email" type="email" bind:value={email}
              placeholder="nim@mail.ugm.ac.id" class="input" required />
          </div>
          <div>
            <label class="text-sm font-medium mb-1.5 block" for="password">Password</label>
            <input id="password" type="password" bind:value={password}
              placeholder="Minimal 8 karakter" class="input" required />
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
    {/if}
  </div>
</div>
