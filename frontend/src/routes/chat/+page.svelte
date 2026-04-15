<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { chat, askStream } from '$lib/api/client';
  import type { ChatMessage, SessionListItem, SourceRef, ChatSource } from '$lib/api/client';
  import {
    Send, Plus, Trash2, Pencil, Check, X,
    ChevronDown, FileText, Loader2, MessageSquare
  } from 'lucide-svelte';

  let sessions: SessionListItem[] = [];
  let activeSessionId: string | undefined;
  let messages: ChatMessage[] = [];
  let streamingText = '';
  let isStreaming = false;
  let sources: SourceRef[] = [];
  let showSources = false;
  let query = '';
  let categories: string[] = [];
  let selectedCategory = '';
  let error = '';
  let renamingId: string | null = null;
  let renameValue = '';
  let messagesEl: HTMLDivElement;
  let cancelStream: (() => void) | null = null;

  onMount(async () => {
    await loadSessions();
    categories = await chat.categories().catch(() => []);
    const sid = $page.url.searchParams.get('s');
    if (sid) await loadSession(sid);
  });

  async function loadSessions() {
    try { sessions = await chat.sessions(); } catch { sessions = []; }
  }

  async function loadSession(id: string) {
    try {
      const s = await chat.getSession(id);
      messages = s.messages;
      activeSessionId = id;
      goto(`/chat?s=${id}`, { replaceState: true });

      // Restore sources dari pesan assistant terakhir
      const lastAssistant = [...s.messages].reverse().find(m => m.role === 'assistant');
      if (lastAssistant?.sources?.length) {
        sources = (lastAssistant.sources ?? []) as any[];
        showSources = false; // collapsed by default
      } else {
        sources = [];
        showSources = false;
      }

      await scrollBottom();
    } catch { error = 'Gagal memuat sesi'; }
  }

  function newChat() {
    messages = [];
    activeSessionId = undefined;
    sources = [];
    streamingText = '';
    error = '';
    goto('/chat', { replaceState: true });
  }

  async function sendMessage() {
    const q = query.trim();
    if (!q || isStreaming) return;

    query = '';
    error = '';
    sources = [];
    showSources = false;

    messages = [...messages, { role: 'user', content: q, created_at: new Date().toISOString() }];
    await scrollBottom();

    isStreaming = true;
    streamingText = '';

    cancelStream = askStream(
      q,
      activeSessionId,
      (chunk) => { streamingText += chunk; scrollBottom(); },
      async (srcs, sid, _ms) => {
        // Simpan pesan assistant ke list
        messages = [...messages, {
          role: 'assistant',
          content: streamingText,
          created_at: new Date().toISOString(),
          sources: srcs
        }];
        streamingText = '';
        isStreaming = false;
        sources = srcs;
        if (srcs.length) showSources = true;

        // Update session id dan refresh sidebar
        if (sid) {
          const isNew = sid !== activeSessionId;
          activeSessionId = sid;
          if (isNew) {
            goto(`/chat?s=${sid}`, { replaceState: true });
          }
          // Selalu refresh sessions agar history ter-update
          await loadSessions();
        }
        await scrollBottom();
      },
      (err) => { error = err; isStreaming = false; streamingText = ''; }
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); }
  }

  async function deleteSession(id: string) {
    await chat.deleteSession(id);
    sessions = sessions.filter(s => s.id !== id);
    if (activeSessionId === id) newChat();
  }

  async function startRename(s: SessionListItem) {
    renamingId = s.id;
    renameValue = s.title;
    await tick();
  }

  async function saveRename(id: string) {
    if (!renameValue.trim()) { renamingId = null; return; }
    await chat.renameSession(id, renameValue.trim());
    sessions = sessions.map(s => s.id === id ? { ...s, title: renameValue.trim() } : s);
    renamingId = null;
  }

  async function scrollBottom() {
    await tick();
    if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString('id-ID', { day: 'numeric', month: 'short' });
  }

  function showMsgSources(msg: ChatMessage) {
    if (msg.sources?.length) {
      sources = msg.sources as any[];
      showSources = true;
    }
  }

  function autoResize(e: Event) {
    const el = e.target as HTMLTextAreaElement;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 160) + 'px';
  }
</script>

<svelte:head><title>Chat — PMPSTI RAG</title></svelte:head>

<div class="flex h-full overflow-hidden">

  <!-- ── Session sidebar ── -->
  <aside class="hidden lg:flex flex-col w-64 border-r bg-card flex-shrink-0">

    <div class="p-3 border-b">
      <button on:click={newChat}
        class="flex items-center gap-2 w-full px-3 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity font-medium">
        <Plus size={15} />
        Chat baru
      </button>
    </div>

    {#if categories.length > 0}
      <div class="px-3 py-2 border-b">
        <select bind:value={selectedCategory}
          class="w-full text-xs border rounded-md px-2 py-1.5 bg-background text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring">
          <option value="">Semua kategori</option>
          {#each categories as cat}
            <option value={cat}>{cat}</option>
          {/each}
        </select>
      </div>
    {/if}

    <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
      {#if sessions.length === 0}
        <p class="text-xs text-muted-foreground text-center py-6">Belum ada riwayat chat</p>
      {/if}

      {#each sessions as s (s.id)}
        <!-- Gunakan class biasa, bukan class:hover: yang tidak valid -->
        <div
          class="group flex items-center gap-1 px-2 py-1.5 rounded-lg cursor-pointer text-sm transition-colors {activeSessionId === s.id ? 'bg-accent text-accent-foreground' : 'hover:bg-muted'}"
          on:click={() => loadSession(s.id)}
          on:keydown={(e) => e.key === 'Enter' && loadSession(s.id)}
          role="button"
          tabindex="0"
        >
          {#if renamingId === s.id}
            <input
              bind:value={renameValue}
              on:keydown={(e) => {
                if (e.key === 'Enter') saveRename(s.id);
                if (e.key === 'Escape') renamingId = null;
              }}
              on:click|stopPropagation
              class="flex-1 text-xs bg-background border rounded px-1.5 py-0.5 focus:outline-none focus:ring-1 focus:ring-ring min-w-0"
              autofocus
            />
            <button on:click|stopPropagation={() => saveRename(s.id)}
              class="p-0.5 hover:text-primary shrink-0">
              <Check size={12} />
            </button>
            <button on:click|stopPropagation={() => renamingId = null}
              class="p-0.5 hover:text-destructive shrink-0">
              <X size={12} />
            </button>
          {:else}
            <MessageSquare size={13} class="shrink-0 text-muted-foreground" />
            <span class="flex-1 truncate text-xs">{s.title || 'Sesi tanpa judul'}</span>
            <div class="hidden group-hover:flex items-center gap-0.5 shrink-0">
              <button on:click|stopPropagation={() => startRename(s)}
                class="p-0.5 hover:text-primary rounded">
                <Pencil size={11} />
              </button>
              <button on:click|stopPropagation={() => deleteSession(s.id)}
                class="p-0.5 hover:text-destructive rounded">
                <Trash2 size={11} />
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </aside>

  <!-- ── Chat area ── -->
  <div class="flex flex-col flex-1 min-w-0">

    <!-- Messages -->
    <div bind:this={messagesEl} class="flex-1 overflow-y-auto px-4 py-6">

      {#if messages.length === 0 && !isStreaming && !streamingText}
        <div class="flex flex-col items-center justify-center h-full text-center max-w-md mx-auto">
          <div class="w-12 h-12 rounded-2xl bg-muted flex items-center justify-center mb-4">
            <MessageSquare size={22} class="text-muted-foreground" />
          </div>
          <h2 class="text-lg font-semibold mb-1">Tanya dokumen kamu</h2>
          <p class="text-sm text-muted-foreground">
            Ajukan pertanyaan dan AI akan menjawab berdasarkan dokumen yang tersimpan.
          </p>
        </div>
      {/if}

      <div class="max-w-2xl mx-auto space-y-6">

        {#each messages as msg (msg.created_at + msg.role)}
          {#if msg.role === 'user'}
            <div class="flex justify-end">
              <div class="bg-primary text-primary-foreground px-4 py-2.5 rounded-2xl rounded-tr-sm max-w-[80%] text-sm whitespace-pre-wrap">
                {msg.content}
              </div>
            </div>
          {:else}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0 mt-0.5">
                <MessageSquare size={13} class="text-muted-foreground" />
              </div>
              <div class="flex-1">
                <div class="text-sm leading-relaxed prose-chat">
                  {@html msg.content.replace(/\n/g, '<br>')}
                </div>
                {#if msg.sources?.length}
                  <button
                    on:click={() => showMsgSources(msg)}
                    class="mt-2 flex items-center gap-1.5 text-xs text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <FileText size={11} />
                    <span>{msg.sources.length} referensi</span>
                  </button>
                {/if}
              </div>
            </div>
          {/if}
        {/each}

        <!-- Streaming bubble -->
        {#if isStreaming || streamingText}
          <div class="flex gap-3">
            <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0 mt-0.5">
              {#if isStreaming && !streamingText}
                <Loader2 size={13} class="text-muted-foreground animate-spin" />
              {:else}
                <MessageSquare size={13} class="text-muted-foreground" />
              {/if}
            </div>
            <div class="flex-1 text-sm leading-relaxed">
              {#if streamingText}
                {@html streamingText.replace(/\n/g, '<br>')}
                {#if isStreaming}<span class="cursor-blink"></span>{/if}
              {:else}
                <span class="text-muted-foreground text-xs">Sedang mencari & menjawab...</span>
              {/if}
            </div>
          </div>
        {/if}

        {#if error}
          <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5">
            {error}
          </div>
        {/if}

      </div>
    </div>

    <!-- Sources panel -->
    {#if sources.length > 0}
      <div class="border-t bg-muted/30">
        <button
          on:click={() => showSources = !showSources}
          class="flex items-center gap-2 w-full px-4 py-2.5 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
        >
          <FileText size={13} />
          <span>{sources.length} sumber referensi</span>
          <!-- Fix: gunakan class string interpolation, bukan class: pada komponen -->
          <span class="ml-auto transition-transform {showSources ? 'rotate-180' : ''}">
            <ChevronDown size={13} />
          </span>
        </button>

        {#if showSources}
          <div class="px-4 pb-3 grid grid-cols-1 md:grid-cols-2 gap-2 max-h-40 overflow-y-auto">
            {#each sources as src, i}
              <div class="bg-background rounded-lg border p-3 text-xs">
                <div class="flex items-start justify-between gap-2 mb-1">
                  <span class="font-medium line-clamp-1">[{i+1}] {src.title}</span>
                  <span class="text-muted-foreground shrink-0">{(src.score * 100).toFixed(0)}%</span>
                </div>
                <p class="text-muted-foreground line-clamp-2 mb-1.5">{src.snippet}</p>
                <span class="inline-block bg-muted px-1.5 py-0.5 rounded text-muted-foreground">
                  {src.category}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Input -->
    <div class="border-t p-4 bg-background">
      <div class="max-w-2xl mx-auto">
        <div class="flex items-end gap-2 border rounded-xl bg-card px-3 py-2 focus-within:ring-2 focus-within:ring-ring transition-shadow">
          <textarea
            bind:value={query}
            on:keydown={handleKeydown}
            on:input={autoResize}
            placeholder="Tanya sesuatu tentang dokumen..."
            rows="1"
            disabled={isStreaming}
            class="flex-1 resize-none bg-transparent text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed min-h-[24px] max-h-40 py-0.5"
          ></textarea>
          <button
            on:click={sendMessage}
            disabled={!query.trim() || isStreaming}
            class="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg bg-primary text-primary-foreground disabled:opacity-40 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
          >
            {#if isStreaming}
              <Loader2 size={14} class="animate-spin" />
            {:else}
              <Send size={14} />
            {/if}
          </button>
        </div>
        <p class="text-xs text-muted-foreground mt-1.5 text-center">
          Enter kirim · Shift+Enter baris baru
        </p>
      </div>
    </div>

  </div>
</div>