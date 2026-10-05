<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authStore, isLoggedIn, currentUser } from '$lib/stores/auth';
  import { auth } from '$lib/api/client';
  import { MessageSquare, Key, BarChart3, LogOut, User, Menu, X, Moon, Sun, ChevronDown } from 'lucide-svelte';

  let menuOpen = false;
  let userDropdown = false;
  let dark = false;

  const publicRoutes = ['/', '/login', '/register'];

  onMount(async () => {
    dark = document.documentElement.classList.contains('dark');
    if ($isLoggedIn) {
      try { const user = await auth.me(); authStore.setUser(user); }
      catch { authStore.logout(); goto('/login'); }
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

  $: isPublic    = publicRoutes.includes($page.url.pathname);
  $: isAdminUser = $currentUser?.role === 'admin';

  // Reactive per-route active state
  $: isChat      = $page.url.pathname === '/chat'    || $page.url.pathname.startsWith('/chat/');
  $: isKeys      = $page.url.pathname === '/keys'    || $page.url.pathname.startsWith('/keys/');
  $: isAdminPage = $page.url.pathname === '/admin'   || $page.url.pathname.startsWith('/admin/');
  $: isProfile   = $page.url.pathname === '/profile' || $page.url.pathname.startsWith('/profile/');

  const navBase   = 'flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors';
  const navActive = 'bg-white/15 text-white';
  const navIdle   = 'text-white/70 hover:text-white hover:bg-white/10';

  // Mobile drawer nav
  const mobileBase   = 'flex items-center gap-2.5 px-3 py-2.5 rounded-md text-sm transition-colors';
  const mobileActive = 'bg-accent/20 text-foreground font-medium';
  const mobileIdle   = 'text-muted-foreground hover:bg-muted hover:text-foreground';
</script>

{#if isPublic}
  <slot />
{:else if $isLoggedIn}
  <div class="flex flex-col h-screen overflow-hidden bg-background">

    <!-- ── Topbar (desktop + mobile) ── -->
    <header class="bg-[#002147] text-white h-14 flex items-center px-4 gap-4 shrink-0 z-40 border-b border-white/10">

      <!-- Logo -->
      <a href="/chat" class="flex items-center gap-2.5 shrink-0 mr-2">
        <img src="/ugm-logo-white.png" alt="Logo UGM" class="h-8 w-auto" />
        <span class="font-bold text-sm tracking-wide">DTETI</span>
      </a>

      <!-- Nav links — desktop only -->
      <nav class="hidden md:flex items-center gap-1">
        <a href="/chat" class="{navBase} {isChat ? navActive : navIdle}">
          <MessageSquare size={15} /><span>Chat</span>
        </a>
        <a href="/keys" class="{navBase} {isKeys ? navActive : navIdle}">
          <Key size={15} /><span>API Keys</span>
        </a>
        {#if isAdminUser}
          <a href="/admin" class="{navBase} {isAdminPage ? navActive : navIdle}">
            <BarChart3 size={15} /><span>Admin</span>
          </a>
        {/if}
      </nav>

      <!-- Spacer -->
      <div class="flex-1"></div>

      <!-- Right side — desktop -->
      <div class="hidden md:flex items-center gap-1">
        <!-- Dark mode toggle -->
        <button on:click={toggleDark}
          class="p-2 rounded-md text-white/70 hover:text-white hover:bg-white/10 transition-colors"
          title={dark ? 'Light mode' : 'Dark mode'}>
          {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
        </button>

        <!-- User dropdown -->
        <div class="relative">
          <button on:click={() => userDropdown = !userDropdown}
            class="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm text-white/70
                   hover:text-white hover:bg-white/10 transition-colors {isProfile ? 'bg-white/15 text-white' : ''}">
            <User size={15} />
            <span class="max-w-[120px] truncate">{$currentUser?.display_name || $currentUser?.email || 'Akun'}</span>
            <ChevronDown size={13} class="opacity-60" />
          </button>

          {#if userDropdown}
            <!-- Backdrop -->
            <div class="fixed inset-0 z-40" on:click={() => userDropdown = false} role="presentation"></div>
            <!-- Dropdown -->
            <div class="absolute right-0 top-full mt-1.5 w-48 bg-card border rounded-xl shadow-lg z-50 py-1 overflow-hidden">
              <a href="/profile" on:click={() => userDropdown = false}
                class="flex items-center gap-2.5 px-4 py-2.5 text-sm text-foreground hover:bg-muted transition-colors {isProfile ? 'font-medium' : ''}">
                <User size={15} class="text-muted-foreground" />Profil & Pengaturan
              </a>
              <div class="border-t my-1"></div>
              <button on:click={() => { userDropdown = false; logout(); }}
                class="flex items-center gap-2.5 px-4 py-2.5 text-sm text-destructive hover:bg-destructive/10
                       transition-colors w-full text-left">
                <LogOut size={15} />Keluar
              </button>
            </div>
          {/if}
        </div>
      </div>

      <!-- Mobile: hamburger -->
      <button on:click={() => menuOpen = !menuOpen}
        class="md:hidden p-1.5 rounded-md text-white/70 hover:text-white hover:bg-white/10 transition-colors">
        {#if menuOpen}<X size={20} />{:else}<Menu size={20} />{/if}
      </button>
    </header>

    <!-- Mobile drawer -->
    {#if menuOpen}
      <div class="md:hidden fixed inset-0 z-30 bg-black/40 backdrop-blur-sm top-14"
        on:click={() => menuOpen = false} role="presentation">
      </div>
      <div class="md:hidden fixed top-14 left-0 bottom-0 z-40 w-64 bg-card border-r p-2 space-y-0.5 overflow-y-auto">
        <a href="/chat" class="{mobileBase} {isChat ? mobileActive : mobileIdle}"
           on:click={() => menuOpen = false}>
          <MessageSquare size={16} /><span>Chat</span>
        </a>
        <a href="/keys" class="{mobileBase} {isKeys ? mobileActive : mobileIdle}"
           on:click={() => menuOpen = false}>
          <Key size={16} /><span>API Keys</span>
        </a>
        {#if isAdminUser}
          <a href="/admin" class="{mobileBase} {isAdminPage ? mobileActive : mobileIdle}"
             on:click={() => menuOpen = false}>
            <BarChart3 size={16} /><span>Admin</span>
          </a>
        {/if}
        <a href="/profile" class="{mobileBase} {isProfile ? mobileActive : mobileIdle}"
           on:click={() => menuOpen = false}>
          <User size={16} /><span>Profil</span>
        </a>
        <div class="border-t my-1 pt-1">
          <button on:click={() => { menuOpen = false; toggleDark(); }}
            class="{mobileBase} {mobileIdle} w-full text-left">
            {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
            <span>{dark ? 'Light mode' : 'Dark mode'}</span>
          </button>
          <button on:click={() => { menuOpen = false; logout(); }}
            class="{mobileBase} text-destructive hover:bg-destructive/10 w-full text-left">
            <LogOut size={16} /><span>Keluar</span>
          </button>
        </div>
      </div>
    {/if}

    <!-- Main content -->
    <main class="flex-1 overflow-hidden">
      <slot />
    </main>
  </div>
{/if}
