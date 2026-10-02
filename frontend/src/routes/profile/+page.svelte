<script lang="ts">
  import { auth } from '$lib/api/client';
  import { authStore, currentUser } from '$lib/stores/auth';
  import { Loader2, Check } from 'lucide-svelte';

  let displayName = $currentUser?.display_name ?? '';
  let email = $currentUser?.email ?? '';
  let currentPassword = '';
  let newPassword = '';
  let confirmPassword = '';

  let savingProfile = false;
  let savingPassword = false;
  let profileSuccess = false;
  let passwordSuccess = false;
  let profileError = '';
  let passwordError = '';

  async function saveProfile() {
    savingProfile = true;
    profileError = '';
    profileSuccess = false;
    try {
      await auth.updateProfile({
        display_name: displayName || undefined,
        email: email !== $currentUser?.email ? email : undefined
      });
      const user = await auth.me();
      authStore.setUser(user);
      profileSuccess = true;
      setTimeout(() => profileSuccess = false, 3000);
    } catch (e: any) {
      profileError = e.message ?? 'Gagal menyimpan profil';
    } finally {
      savingProfile = false;
    }
  }

  async function savePassword() {
    passwordError = '';
    passwordSuccess = false;
    if (newPassword.length < 8) { passwordError = 'Password baru minimal 8 karakter'; return; }
    if (newPassword !== confirmPassword) { passwordError = 'Konfirmasi password tidak cocok'; return; }
    savingPassword = true;
    try {
      await auth.updatePassword(currentPassword, newPassword);
      currentPassword = '';
      newPassword = '';
      confirmPassword = '';
      passwordSuccess = true;
      setTimeout(() => passwordSuccess = false, 3000);
    } catch (e: any) {
      passwordError = e.message ?? 'Gagal mengubah password';
    } finally {
      savingPassword = false;
    }
  }
</script>

<svelte:head><title>Profil — PMPSTI RAG</title></svelte:head>

<div class="h-full overflow-y-auto p-6">
  <div class="max-w-xl mx-auto space-y-6">
    <div>
      <h1 class="text-xl font-semibold">Profil</h1>
      <p class="text-sm text-muted-foreground mt-0.5">Kelola informasi akun kamu</p>
    </div>

    <!-- Profile section -->
    <div class="bg-card border rounded-xl p-5">
      <h2 class="text-sm font-medium mb-4">Informasi profil</h2>

      {#if profileError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 mb-4">
          {profileError}
        </div>
      {/if}
      {#if profileSuccess}
        <div class="bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300 text-sm rounded-lg px-3 py-2.5 mb-4 flex items-center gap-2">
          <Check size={14} />Profil berhasil disimpan
        </div>
      {/if}

      <form on:submit|preventDefault={saveProfile} class="space-y-4">
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block" for="dname">Nama tampilan</label>
          <input id="dname" type="text" bind:value={displayName} placeholder="Nama kamu" class="input" />
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block" for="pemail">Email</label>
          <input id="pemail" type="email" bind:value={email} class="input" required />
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block">Role</label>
          <input value={$currentUser?.role ?? ''} disabled class="input opacity-60 cursor-not-allowed" />
        </div>
        <button type="submit" disabled={savingProfile}
          class="flex items-center gap-2 bg-primary text-primary-foreground text-sm font-medium px-4 py-2 rounded-lg hover:opacity-90 disabled:opacity-50 transition-opacity">
          {#if savingProfile}<Loader2 size={14} class="animate-spin" />{/if}
          Simpan perubahan
        </button>
      </form>
    </div>

    <!-- Password section -->
    <div class="bg-card border rounded-xl p-5">
      <h2 class="text-sm font-medium mb-4">Ubah password</h2>

      {#if passwordError}
        <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 mb-4">
          {passwordError}
        </div>
      {/if}
      {#if passwordSuccess}
        <div class="bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 text-green-700 dark:text-green-300 text-sm rounded-lg px-3 py-2.5 mb-4 flex items-center gap-2">
          <Check size={14} />Password berhasil diubah
        </div>
      {/if}

      <form on:submit|preventDefault={savePassword} class="space-y-4">
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block" for="cpwd">Password saat ini</label>
          <input id="cpwd" type="password" bind:value={currentPassword} placeholder="••••••••" class="input" required />
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block" for="npwd">Password baru</label>
          <input id="npwd" type="password" bind:value={newPassword} placeholder="Minimal 8 karakter" class="input" required />
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground mb-1.5 block" for="cpwd2">Konfirmasi password baru</label>
          <input id="cpwd2" type="password" bind:value={confirmPassword} placeholder="Ulangi password baru" class="input" required />
        </div>
        <button type="submit" disabled={savingPassword}
          class="flex items-center gap-2 bg-primary text-primary-foreground text-sm font-medium px-4 py-2 rounded-lg hover:opacity-90 disabled:opacity-50 transition-opacity">
          {#if savingPassword}<Loader2 size={14} class="animate-spin" />{/if}
          Ubah password
        </button>
      </form>
    </div>
  </div>
</div>

