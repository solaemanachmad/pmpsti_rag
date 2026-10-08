<script lang="ts">
  import { onMount } from 'svelte';
  import { faq } from '$lib/api/client';
  import type { FaqPin, TopQuery } from '$lib/api/client';
  import { Loader2, Pin, PinOff, TrendingUp, RefreshCw, GripVertical, Trash2 } from 'lucide-svelte';

  let pins: FaqPin[]    = [];
  let topQ: TopQuery[]  = [];
  let loading           = false;
  let saving            = false;
  let error             = '';
  let success           = '';
  let newQuestion       = '';

  onMount(load);

  async function load() {
    loading = true; error = '';
    try {
      [pins, topQ] = await Promise.all([faq.pins(), faq.topQueries()]);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { loading = false; }
  }

  // Apakah query sudah di-pin?
  function isPinned(q: string): boolean {
    return pins.some(p => p.question === q);
  }

  async function pinQuestion(question: string) {
    saving = true; error = ''; success = '';
    try {
      await faq.pin(question);
      success = 'Pertanyaan berhasil di-pin';
      await load();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { saving = false; }
  }

  async function unpinQuestion(id: number) {
    saving = true; error = ''; success = '';
    try {
      await faq.unpin(id);
      success = 'Pin dihapus';
      await load();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { saving = false; }
  }

  async function addManual() {
    const q = newQuestion.trim();
    if (!q) return;
    saving = true; error = ''; success = '';
    try {
      await faq.pin(q);
      newQuestion = '';
      success = 'Pertanyaan manual berhasil ditambahkan';
      await load();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { saving = false; }
  }

  function formatDate(s: string) {
    return new Date(s).toLocaleDateString('id-ID', { day: 'numeric', month: 'short', year: 'numeric' });
  }
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-xl font-bold">Kelola FAQ</h1>
      <p class="text-sm text-muted-foreground mt-0.5">
        Pin pertanyaan yang ditampilkan di halaman depan. Maks. 8 pertanyaan (pinned + otomatis).
      </p>
    </div>
    <button on:click={load} disabled={loading}
      class="flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-lg border
             hover:bg-muted transition-colors disabled:opacity-50">
      <RefreshCw size={14} class={loading ? 'animate-spin' : ''} /> Refresh
    </button>
  </div>

  {#if error}
    <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-4 py-2.5">
      {error}
    </div>
  {/if}
  {#if success}
    <div class="bg-green-500/10 border border-green-500/20 text-green-700 dark:text-green-400 text-sm rounded-lg px-4 py-2.5">
      {success}
    </div>
  {/if}

  {#if loading}
    <div class="flex items-center gap-2 text-muted-foreground py-8 justify-center">
      <Loader2 size={18} class="animate-spin" /> Memuat…
    </div>
  {:else}

    <!-- ── Pinned FAQ ── -->
    <section class="border rounded-xl overflow-hidden">
      <div class="px-4 py-3 bg-muted/40 border-b flex items-center justify-between">
        <div class="flex items-center gap-2 font-semibold text-sm">
          <Pin size={14} class="text-[#0055A5]" />
          FAQ Aktif (Pinned) — {pins.length} pertanyaan
        </div>
        <span class="text-xs text-muted-foreground">
          Ditampilkan paling atas di halaman depan
        </span>
      </div>

      {#if pins.length === 0}
        <p class="px-4 py-6 text-sm text-muted-foreground text-center">
          Belum ada pertanyaan yang di-pin. Pin dari top queries di bawah, atau tambah manual.
        </p>
      {:else}
        <ul class="divide-y">
          {#each pins as p}
            <li class="flex items-center gap-3 px-4 py-3">
              <GripVertical size={14} class="text-muted-foreground/40 shrink-0" />
              <span class="flex-1 text-sm">{p.question}</span>
              <span class="text-xs text-muted-foreground shrink-0">{formatDate(p.created_at)}</span>
              <button
                on:click={() => unpinQuestion(p.id)}
                disabled={saving}
                title="Hapus pin"
                class="shrink-0 p-1.5 rounded-lg text-muted-foreground hover:text-destructive
                       hover:bg-destructive/10 transition-colors disabled:opacity-50">
                <Trash2 size={14} />
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      <!-- Tambah pertanyaan manual -->
      <div class="px-4 py-3 border-t bg-muted/20">
        <p class="text-xs font-medium text-muted-foreground mb-2">Tambah pertanyaan manual:</p>
        <div class="flex gap-2">
          <input
            bind:value={newQuestion}
            on:keydown={(e) => e.key === 'Enter' && addManual()}
            placeholder="Ketik pertanyaan lalu Enter…"
            class="flex-1 text-sm px-3 py-1.5 rounded-lg border bg-background
                   focus:outline-none focus:ring-2 focus:ring-[#0055A5]/30" />
          <button
            on:click={addManual}
            disabled={saving || !newQuestion.trim()}
            class="flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-lg
                   bg-[#0055A5] text-white hover:bg-[#002147] transition-colors
                   disabled:opacity-50">
            {#if saving}<Loader2 size={13} class="animate-spin" />{:else}<Pin size={13} />{/if}
            Pin
          </button>
        </div>
      </div>
    </section>

    <!-- ── Top Queries (otomatis) ── -->
    <section class="border rounded-xl overflow-hidden">
      <div class="px-4 py-3 bg-muted/40 border-b flex items-center gap-2 font-semibold text-sm">
        <TrendingUp size={14} class="text-amber-500" />
        Top Queries Otomatis — pilih untuk di-pin
      </div>

      {#if topQ.length === 0}
        <p class="px-4 py-6 text-sm text-muted-foreground text-center">
          Belum ada query log. Data akan muncul setelah mahasiswa mulai bertanya.
        </p>
      {:else}
        <ul class="divide-y">
          {#each topQ as q}
            {@const pinned = isPinned(q.query_text)}
            <li class="flex items-center gap-3 px-4 py-2.5
                       {pinned ? 'opacity-50' : ''}">
              <span class="shrink-0 min-w-[2.5rem] text-xs font-bold text-amber-600
                           bg-amber-50 dark:bg-amber-900/20 px-2 py-0.5 rounded-full text-center">
                {q.count}×
              </span>
              <span class="flex-1 text-sm">{q.query_text}</span>
              <span class="text-xs text-muted-foreground shrink-0 hidden sm:block">
                Terakhir: {formatDate(q.last_asked)}
              </span>
              <button
                on:click={() => {
                const pin = pins.find(p => p.question === q.query_text);
                if (pinned && pin) unpinQuestion(pin.id);
                else pinQuestion(q.query_text);
              }}
                disabled={saving}
                title={pinned ? 'Hapus pin' : 'Pin pertanyaan ini'}
                class="shrink-0 flex items-center gap-1 text-xs px-2.5 py-1 rounded-lg border
                       transition-colors disabled:opacity-50
                       {pinned
                         ? 'border-destructive/30 text-destructive hover:bg-destructive/10'
                         : 'border-[#0055A5]/30 text-[#0055A5] hover:bg-[#0055A5]/10'}">
                {#if pinned}
                  <PinOff size={12} /> Unpin
                {:else}
                  <Pin size={12} /> Pin
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

  {/if}
</div>
