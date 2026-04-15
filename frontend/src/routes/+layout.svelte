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

  const publicRoutes = ['/login', '/register'];

  onMount(async () => {
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

  // Class helper — tidak pakai @apply, langsung string
  const navBase = 'flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer w-full';
  const navActive = 'bg-accent text-accent-foreground font-medium';

  function navClass(path: string) {
    const active = $page.url.pathname.startsWith(path);
    return `${navBase} ${active ? navActive : ''}`;
  }
</script>

{#if isPublic}
  <slot />
{:else if $isLoggedIn}
  <div class="flex h-screen overflow-hidden bg-background">

    <!-- Sidebar desktop -->
    <aside class="hidden md:flex w-56 flex-col border-r bg-card">
      <div class="flex items-center gap-2 px-4 py-5 border-b">
        <div class="w-7 h-7 rounded-lg bg-primary flex items-center justify-center">
          <MessageSquare size={14} class="text-primary-foreground" />
        </div>
        <span class="font-semibold text-sm">PMPSTI RAG</span>
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
            <BarChart3 size={16} /><span>Dashboard</span>
          </a>
        {/if}
      </nav>

      <div class="p-2 border-t space-y-0.5">
        <a href="/profile" class={navClass('/profile')}>
          <User size={16} />
          <span class="truncate">{$currentUser?.display_name || $currentUser?.email || 'Profil'}</span>
        </a>
        <button on:click={toggleDark}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer w-full text-left">
          {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
          <span>{dark ? 'Light mode' : 'Dark mode'}</span>
        </button>
        <button on:click={logout}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-destructive hover:bg-destructive/10 transition-colors cursor-pointer w-full text-left">
          <LogOut size={16} /><span>Keluar</span>
        </button>
      </div>
    </aside>

    <!-- Topbar mobile -->
    <div class="md:hidden fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-4 h-14 border-b bg-background">
      <div class="flex items-center gap-2">
        <div class="w-6 h-6 rounded-md bg-primary flex items-center justify-center">
          <MessageSquare size={12} class="text-primary-foreground" />
        </div>
        <span class="font-semibold text-sm">PMPSTI RAG</span>
      </div>
      <button on:click={() => menuOpen = !menuOpen} class="p-1.5 rounded-md hover:bg-muted">
        {#if menuOpen}<X size={18} />{:else}<Menu size={18} />{/if}
      </button>
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
            <BarChart3 size={16} /><span>Dashboard</span>
          </a>
        {/if}
        <button on:click={logout}
          class="flex items-center gap-2.5 px-3 py-2 rounded-md text-sm text-destructive hover:bg-destructive/10 transition-colors cursor-pointer w-full text-left">
          <LogOut size={16} /><span>Keluar</span>
        </button>
      </div>
    {/if}

    <main class="flex-1 overflow-hidden md:mt-0 mt-14">
      <slot />
    </main>
  </div>
{/if}