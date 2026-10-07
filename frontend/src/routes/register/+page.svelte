<script lang="ts">
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import { auth } from '$lib/api/client';
  import { Loader2, MailCheck } from 'lucide-svelte';

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

<svelte:head><title>Daftar — PMPSTI</title></svelte:head>

<div class="fixed top-4 right-4 z-50">
  <ThemeToggle />
</div>

<div class="min-h-screen flex flex-col bg-background">
  <div class="bg-[#002147] h-1.5 w-full"></div>
  <div class="bg-[#0055A5] h-0.5 w-full mb-8"></div>

  <div class="flex-1 flex flex-col items-center justify-center px-4 pb-12">
    <!-- Branding -->
    <div class="flex flex-col items-center gap-3 mb-8">
      <picture>
        <source srcset="/ugm-logo-white.png" media="(prefers-color-scheme: dark)" />
        <img src="/ugm-logo-navy.png" alt="Logo UGM" class="w-16 h-16"
             onerror="this.src='/ugm-logo-white.png'" />
      </picture>
      <div class="text-center">
        <div class="font-bold text-lg text-foreground">PMPSTI</div>
        <div class="text-xs text-muted-foreground">Asisten Akademik — Universitas Gadjah Mada</div>
      </div>
    </div>

    {#if registered}
      <!-- ── Sukses ── -->
      <div class="w-full max-w-sm bg-card border rounded-xl p-6 shadow-sm text-center space-y-3">
        <div class="w-14 h-14 rounded-full bg-[#0055A5]/10 flex items-center justify-center mx-auto">
          <MailCheck size={26} class="text-[#0055A5]" />
        </div>
        <h1 class="text-base font-semibold">Cek email kamu</h1>
        <p class="text-sm text-muted-foreground leading-relaxed">
          Link verifikasi sudah dikirim ke<br>
          <span class="font-semibold text-foreground">{email}</span>
        </p>
        <p class="text-xs text-muted-foreground">
          Klik link tersebut untuk mengaktifkan akun. Berlaku <strong>24 jam</strong>.
        </p>
        <a href="/login"
          class="btn-primary w-full mt-1 bg-[#0055A5] hover:bg-[#002147]">
          Pergi ke halaman masuk
        </a>
      </div>

    {:else}
      <!-- ── Form ── -->
      <div class="w-full max-w-sm bg-card border rounded-xl p-6 shadow-sm">
        <h1 class="text-base font-semibold mb-1">Buat akun</h1>
        <p class="text-xs text-muted-foreground mb-5">
          Khusus untuk email <span class="font-semibold text-foreground">@mail.ugm.ac.id</span>
        </p>

        {#if error}
          <div class="bg-destructive/10 border border-destructive/20 text-destructive
                      text-xs rounded-lg px-3 py-2.5 mb-4 leading-relaxed">
            {error}
          </div>
        {/if}

        <form on:submit|preventDefault={handleRegister} class="space-y-4">
          <div>
            <label class="text-xs font-medium mb-1.5 block text-muted-foreground" for="name">
              Nama lengkap <span class="text-muted-foreground/60">(opsional)</span>
            </label>
            <input id="name" type="text" bind:value={display_name}
              placeholder="Nama kamu" class="input" autocomplete="name" />
          </div>
          <div>
            <label class="text-xs font-medium mb-1.5 block text-muted-foreground" for="email">
              Email UGM
            </label>
            <input id="email" type="email" bind:value={email}
              placeholder="nim@mail.ugm.ac.id" class="input" autocomplete="email" required />
          </div>
          <div>
            <label class="text-xs font-medium mb-1.5 block text-muted-foreground" for="password">
              Password
            </label>
            <input id="password" type="password" bind:value={password}
              placeholder="Minimal 8 karakter" class="input" autocomplete="new-password" required />
          </div>
          <button type="submit" disabled={loading}
            class="btn-primary w-full mt-1 bg-[#0055A5] hover:bg-[#002147]">
            {#if loading}<Loader2 size={15} class="animate-spin" />{/if}
            Daftar
          </button>
        </form>

        <p class="text-xs text-center text-muted-foreground mt-5">
          Sudah punya akun?
          <a href="/login" class="text-[#0055A5] font-semibold hover:underline">Masuk</a>
        </p>
      </div>
    {/if}

    <a href="/" class="mt-5 text-xs text-muted-foreground hover:text-foreground transition-colors">
      ← Kembali ke halaman utama
    </a>
  </div>
</div>
