<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore, isLoggedIn, currentUser } from '$lib/stores/auth';
  import { auth } from '$lib/api/client';
  import { MessageSquare, Key, BarChart3, LogOut, User, Menu, X, Moon, Sun } from 'lucide-svelte';

  let menuOpen = false;
  let dark = false;

  const publicRoutes = ['/', '/login', '/register'];

  onMount(async () => {
    // Sync dark state from DOM (set by inline script in app.html)
    dark = document.documentElement.classList.contains('dark');

    if ($isLoggedIn) {
      try {
        const user = await auth.me();
        authStore.setUser(user);
      } catch {
        authStore.logout();
        goto('/login');
      }
    } else if (!publicRoutes.includes($page.url.pathname)) {
      goto('/login');
    }
  });

  $: if (typeof window !== 'undefined' && !$isLoggedIn && !publicRoutes.includes($page.url.pathname)) {
    goto('/login');
  }

  function toggleDark() {
    dark = !dark;
    document.documentElement.classList.toggle('dark', dark);
    localStorage.setItem('theme', dark ? 'dark' : 'light');
  }

  function logout() {
    authStore.logout();
    goto('/login');
  }

  $: isPublic = publicRoutes.includes($page.url.pathname);
  $: isAdmin = $currentUser?.role === 'admin';

  const navBase = 'flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent/20 hover:text-foreground transition-colors cursor-pointer w-full';
  const navActive = 'bg-accent/20 text-foreground font-medium';

  function navClass(path: string) {
    const active = $page.url.pathname.startsWith(path);
    return `${navBase} ${active ? navActive : ''}`;
  }
</script>

{#if isPublic}
  <!-- Dark mode toggle untuk halaman publik (pojok kanan atas) -->
  <div class="fixed top-4 right-4 z-50">
    <button on:click={toggleDark}
      title="{dark ? 'Light mode' : 'Dark mode'}"
      class="p-2 rounded-full bg-background/80 backdrop-blur border border-border
             text-muted-foreground hover:text-foreground hover:bg-muted
             transition-all shadow-sm">
      {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
    </button>
  </div>
  <slot />
{:else if $isLoggedIn}
  <div class="flex h-screen overflow-hidden bg-background">

    <!-- Sidebar desktop -->
    <aside class="hidden md:flex w-56 flex-col border-r bg-card">
      <div class="flex items-center gap-2.5 px-4 py-4 border-b bg-[#002147]">
        <img src="/ugm-logo-white.png" alt="Logo UGM" class="w-8 h-8 shrink-0" />
        <div class="leading-tight">
          <div class="font-semibold text-sm text-white">PMPSTI</div>
          <div class="text-[10px] text-white/50 leading-none">Universitas Gadjah Mada</div>
        </div>
      </div>

      <nav class="flex-1 p-2 space-y-0.5 overflow-y-auto">
        <a href="/chat" class={navClass('/chat')}>
          <MessageSquare size={16} /><span>Chat</span>
        </a>
        <a href="/keys" class={navClass('/keys')}>
          <Key size={16} /><span>API Keys</span>
        </a>
        {#if isAdmin}
          <a href="/admin" class={navClass('/admin')}>
            <BarChart3 size={16} /><span>Admin</span>
          </a>
        {/if}
      </nav>

      <div class="p-2 border-t space-y-0.5">
        <a href="/profile" class={navClass('/profile')}>
          <User size={16} />
          <span class="truncate">{$currentUser?.display_name || $currentUser?.email || 'Profil'}</span>
        </a>
        <button on:click={toggleDark}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground
                 hover:bg-muted hover:text-foreground transition-colors cursor-pointer w-full text-left">
          {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
          <span>{dark ? 'Light mode' : 'Dark mode'}</span>
        </button>
        <button on:click={logout}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-destructive
                 hover:bg-destructive/10 transition-colors cursor-pointer w-full text-left">
          <LogOut size={16} /><span>Keluar</span>
        </button>
      </div>
    </aside>

    <!-- Topbar mobile -->
    <div class="md:hidden fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-4 h-14 border-b bg-background">
      <div class="flex items-center gap-2">
        <img src="/ugm-logo-white.png" alt="Logo UGM" class="w-7 h-7 shrink-0" />
        <div class="font-semibold text-sm">PMPSTI</div>
      </div>
      <div class="flex items-center gap-1">
        <button on:click={toggleDark}
          class="p-2 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
          {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
        </button>
        <button on:click={() => menuOpen = !menuOpen} class="p-1.5 rounded-md hover:bg-muted">
          {#if menuOpen}<X size={18} />{:else}<Menu size={18} />{/if}
        </button>
      </div>
    </div>

    {#if menuOpen}
      <div class="md:hidden fixed inset-0 z-40 bg-background/80 backdrop-blur-sm"
        on:click={() => menuOpen = false}
        role="presentation">
      </div>
      <div class="md:hidden fixed top-14 left-0 bottom-0 z-40 w-64 bg-card border-r p-2 space-y-0.5 overflow-y-auto">
        <a href="/chat" class={navClass('/chat')} on:click={() => menuOpen = false}>
          <MessageSquare size={16} /><span>Chat</span>
        </a>
        <a href="/keys" class={navClass('/keys')} on:click={() => menuOpen = false}>
          <Key size={16} /><span>API Keys</span>
        </a>
        {#if isAdmin}
          <a href="/admin" class={navClass('/admin')} on:click={() => menuOpen = false}>
            <BarChart3 size={16} /><span>Admin</span>
          </a>
        {/if}
        <a href="/profile" class={navClass('/profile')} on:click={() => menuOpen = false}>
          <User size={16} /><span>Profil</span>
        </a>
        <button on:click={logout}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-destructive
                 hover:bg-destructive/10 transition-colors cursor-pointer w-full text-left">
          <LogOut size={16} /><span>Keluar</span>
        </button>
      </div>
    {/if}

    <main class="flex-1 overflow-hidden md:mt-0 mt-14">
      <slot />
    </main>
  </div>
{/if}
