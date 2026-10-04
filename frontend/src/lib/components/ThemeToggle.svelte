<script lang="ts">
  import { onMount } from 'svelte';
  import { Moon, Sun } from 'lucide-svelte';

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
  class="p-2 rounded-full bg-background/80 backdrop-blur border border-border
         text-muted-foreground hover:text-foreground hover:bg-muted
         transition-all shadow-sm"
>
  {#if dark}<Sun size={16} />{:else}<Moon size={16} />{/if}
</button>
