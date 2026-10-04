<script lang="ts">
  import { onMount } from 'svelte';
  import { Moon, Sun } from 'lucide-svelte';

  /** Jika true, pakai style untuk header gelap (teks putih) */
  export let onDark = false;

  let dark = false;

  onMount(() => {
    dark = document.documentElement.classList.contains('dark');
  });

  function toggle() {
    dark = !dark;
    document.documentElement.classList.toggle('dark', dark);
    try { localStorage.setItem('theme', dark ? 'dark' : 'light'); } catch {}
  }
</script>

<button
  on:click={toggle}
  title={dark ? 'Light mode' : 'Dark mode'}
  class={onDark
    ? 'p-2 rounded-lg text-white/70 hover:text-white bg-white/10 hover:bg-white/20 transition-colors'
    : 'p-2 rounded-full bg-background/80 backdrop-blur border border-border text-muted-foreground hover:text-foreground hover:bg-muted transition-all shadow-sm'
  }
>
  {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
</button>
