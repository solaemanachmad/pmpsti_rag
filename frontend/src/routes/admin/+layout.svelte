<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { currentUser, authReady } from '$lib/stores/auth';
  import { BarChart3, Users, FileText, List } from 'lucide-svelte';

  const navItems = [
    { href: '/admin',           label: 'Dashboard', icon: BarChart3 },
    { href: '/admin/users',     label: 'Pengguna',  icon: Users },
    { href: '/admin/documents', label: 'Dokumen',   icon: FileText },
    { href: '/admin/logs',      label: 'Log Query', icon: List },
  ];

  onMount(() => {
    const unsub = authReady.subscribe(ready => {
      if (!ready) return;
      unsub();
      let u: import('$lib/api/client').UserPublic | null = null;
      const u2 = currentUser.subscribe(v => { u = v; }); u2();
      if (!u) { goto('/login'); return; }
      if ((u as any).role !== 'admin') goto('/chat');
    });
  });

  $: active = $page.url.pathname;
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Sub-nav -->
  <div class="border-b bg-background px-4 shrink-0">
    <div class="max-w-5xl mx-auto flex items-center gap-1">
      <div class="flex items-center gap-0.5 py-1">
        {#each navItems as item}
          <a href={item.href}
             class="flex items-center gap-1.5 px-3 py-2 text-sm font-medium rounded-md transition-colors
                    {active === item.href
                      ? 'text-[#0055A5] bg-[#0055A5]/8'
                      : 'text-muted-foreground hover:text-foreground hover:bg-muted'}">
            <svelte:component this={item.icon} size={14} />
            {item.label}
          </a>
        {/each}
      </div>
    </div>
  </div>

  <!-- Page content -->
  <div class="flex-1 overflow-y-auto">
    <div class="max-w-5xl mx-auto px-4 py-6">
      <slot />
    </div>
  </div>
</div>
