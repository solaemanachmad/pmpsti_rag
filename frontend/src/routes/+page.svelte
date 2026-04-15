<script lang="ts">
  import { goto } from '$app/navigation';
  import { isLoggedIn } from '$lib/stores/auth';
  import { onMount } from 'svelte';

  onMount(() => {
    goto($isLoggedIn ? '/chat' : '/login');
  });
</script>