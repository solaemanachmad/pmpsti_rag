<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { chat, askStream } from '$lib/api/client';
  import type { ChatMessage, ChatSource, SessionListItem, SourceRef } from '$lib/api/client';
  import {
    Send, Plus, Trash2, Pencil, Check, X,
    ChevronDown, FileText, Loader2, MessageSquare
  } from 'lucide-svelte';

  // State
  let sessions: SessionListItem[] = [];
  let activeSessionId: string | undefined;
  let messages: ChatMessage[] = [];
  let streamingText = '';
  let isStreaming = false;
  let query = '';
  let categories: string[] = [];
  let selectedCategory = '';
  let error = '';

  // Rename state
  let renamingId: string | null = null;
  let renameValue = '';

  let messagesEl: HTMLDivElement;
  let inputEl: HTMLTextAreaElement;
  let cancelStream: (() => void) | null = null;

  onMount(async () => {
    await loadSessions();
    categories = await chat.categories().catch(() => []);

    // Load session from URL param
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
      await scrollBottom();
    } catch { error = 'Gagal memuat sesi'; }
  }

  function newChat() {
    messages = [];
    activeSessionId = undefined;
    streamingText = '';
    goto('/chat', { replaceState: true });
  }

  async function sendMessage() {
    const q = query.trim();
    if (!q || isStreaming) return;

    query = '';
    error = '';
    // Tambah pesan user optimistically
    messages = [...messages, { role: 'user', content: q, created_at: new Date().toISOString() }];
    await scrollBottom();

    isStreaming = true;
    streamingText = '';

    cancelStream = askStream(
      q,
      activeSessionId,
      (chunk) => {
        streamingText += chunk;
        scrollBottom();
      },
      async (srcs, sid, _ms) => {
        // Selesai streaming
        messages = [
          ...messages,
          {
            role: 'assistant',
            content: streamingText,
            created_at: new Date().toISOString(),
            sources: srcs as any
          }
        ];
        streamingText = '';
        isStreaming = false;

        if (sid && sid !== activeSessionId) {
          activeSessionId = sid;
          goto(`/chat?s=${sid}`, { replaceState: true });
          await loadSessions();
        }
        await scrollBottom();
      },
      (err) => {
        error = err;
        isStreaming = false;
        streamingText = '';
      }
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
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
    const d = new Date(iso);
    return d.toLocaleDateString('id-ID', { day: 'numeric', month: 'short' });
  }

  // Auto-resize textarea
  function autoResize(e: Event) {
    const el = e.target as HTMLTextAreaElement;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 160) + 'px';
  }
  function getMsgSources(msg: ChatMessage): ChatSource[] {
    return (msg.sources ?? []) as ChatSource[];
  }

</script>

<svelte:head><title>Chat — PMPSTI RAG</title></svelte:head>

<div class="flex h-full overflow-hidden">
  <!-- Session sidebar -->
  <aside class="hidden lg:flex flex-col w-64 border-r bg-card flex-shrink-0">
    <div class="p-3 border-b">
      <button on:click={newChat}
        class="flex items-center gap-2 w-full px-3 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-opacity font-medium">
        <Plus size={15} />
        Chat baru
      </button>
    </div>

    <!-- Category filter -->
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

    <!-- Sessions list -->
    <div class="flex-1 overflow-y-auto p-2 space-y-0.5">
      {#if sessions.length === 0}
        <p class="text-xs text-muted-foreground text-center py-6">Belum ada riwayat chat</p>
      {/if}
      {#each sessions as s (s.id)}
        <div
          class="group flex items-center gap-1 px-2 py-1.5 rounded-lg cursor-pointer text-sm transition-colors"
          class:bg-accent={activeSessionId === s.id}
          class:text-accent-foreground={activeSessionId === s.id}
          class:hover:bg-muted={activeSessionId !== s.id}
          on:click={() => loadSession(s.id)}
        >
          {#if renamingId === s.id}
            <input
              bind:value={renameValue}
              on:keydown={(e) => { if (e.key === 'Enter') saveRename(s.id); if (e.key === 'Escape') renamingId = null; }}
              class="flex-1 text-xs bg-background border rounded px-1.5 py-0.5 focus:outline-none focus:ring-1 focus:ring-ring min-w-0"
              on:click|stopPropagation
              autofocus
            />
            <button on:click|stopPropagation={() => saveRename(s.id)} class="p-0.5 hover:text-primary shrink-0">
              <Check size={12} />
            </button>
            <button on:click|stopPropagation={() => renamingId = null} class="p-0.5 hover:text-destructive shrink-0">
              <X size={12} />
            </button>
          {:else}
            <MessageSquare size={13} class="shrink-0 text-muted-foreground" />
            <span class="flex-1 truncate text-xs">{s.title || 'Sesi tanpa judul'}</span>
            <span class="text-xs text-muted-foreground hidden group-hover:hidden shrink-0">{formatDate(s.updated_at)}</span>
            <div class="hidden group-hover:flex items-center gap-0.5 shrink-0">
              <button on:click|stopPropagation={() => startRename(s)} class="p-0.5 hover:text-primary rounded">
                <Pencil size={11} />
              </button>
              <button on:click|stopPropagation={() => deleteSession(s.id)} class="p-0.5 hover:text-destructive rounded">
                <Trash2 size={11} />
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </aside>

  <!-- Chat area -->
  <div class="flex flex-col flex-1 min-w-0">
    <!-- Messages -->
    <div bind:this={messagesEl} class="flex-1 overflow-y-auto px-4 py-6">
      {#if messages.length === 0 && !isStreaming}
        <!-- Empty state -->
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
              <div class="bg-primary text-primary-foreground px-4 py-2.5 rounded-2xl rounded-tr-sm max-w-[80%] text-sm">
                {msg.content}
              </div>
            </div>
          {:else}
            {@const msgSources = getMsgSources(msg)}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0 mt-0.5">
                <MessageSquare size={13} class="text-muted-foreground" />
              </div>
              <div class="flex-1 min-w-0">
                <!-- Jawaban -->
                <div class="prose-chat text-sm leading-relaxed">
                  {@html msg.content.replace(/\n/g, '<br>')}
                </div>

                <!-- Referensi per pesan — selalu ada selama sesi hidup atau dimuat ulang -->
                {#if msgSources.length > 0}
                  <details class="mt-3 group">
                    <summary class="flex items-center gap-1.5 text-xs text-muted-foreground
                                   hover:text-foreground cursor-pointer select-none list-none
                                   w-fit">
                      <FileText size={12} />
                      <span>{msgSources.length} referensi</span>
                      <span class="transition-transform group-open:rotate-180">
                        <ChevronDown size={11} />
                      </span>
                    </summary>
                    <div class="mt-2 grid grid-cols-1 sm:grid-cols-2 gap-2">
                      {#each msgSources as src, i}
                        {@const hasUrl = src.source_url && src.source_url.startsWith('http')}
                        {@const domain = hasUrl ? (() => { try { return new URL(src.source_url).hostname } catch { return '' } })() : ''}
                        <div class="rounded-lg border bg-muted/30 text-xs overflow-hidden
                                    {hasUrl ? 'hover:border-primary/40 transition-colors' : ''}">
                          <div class="flex items-start justify-between gap-2 px-3 pt-2.5 pb-1">
                            <span class="font-medium leading-snug line-clamp-2 flex-1">
                              [{i+1}] {src.title}
                            </span>
                            <span class="text-muted-foreground shrink-0 tabular-nums text-[10px]">
                              {(src.score * 100).toFixed(0)}%
                            </span>
                          </div>
                          <p class="text-muted-foreground line-clamp-2 px-3 pb-1.5 leading-relaxed">
                            {src.snippet}
                          </p>
                          <div class="flex items-center justify-between gap-2 px-3 pb-2">
                            <span class="bg-background px-1.5 py-0.5 rounded border text-muted-foreground">
                              {src.category || 'dokumen'}
                            </span>
                            {#if hasUrl}
                              <a href={src.source_url}
                                 target="_blank"
                                 rel="noopener noreferrer"
                                 class="flex items-center gap-1 text-primary hover:underline shrink-0"
                                 title={src.source_url}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"
                                     viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                     stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                                  <polyline points="15 3 21 3 21 9"/>
                                  <line x1="10" y1="14" x2="21" y2="3"/>
                                </svg>
                                <span class="max-w-[110px] truncate">{domain}</span>
                              </a>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </details>
                {/if}
              </div>
            </div>
          {/if}
        {/each}

        <!-- Streaming message -->
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
                <span>{@html streamingText.replace(/\n/g, '<br>')}</span>
                {#if isStreaming}<span class="cursor-blink"></span>{/if}
              {:else}
                <span class="text-muted-foreground text-xs">Sedang mencari & menjawab...</span>
              {/if}
            </div>
          </div>
        {/if}

        {#if error}
          <div class="bg-destructive/10 border border-destructive/20 text-destructive text-sm rounded-lg px-3 py-2.5 max-w-2xl">
            {error}
          </div>
        {/if}
      </div>
    </div>



    <!-- Input -->
    <div class="border-t p-4 bg-background">
      <div class="max-w-2xl mx-auto">
        <div class="flex items-end gap-2 border rounded-xl bg-card px-3 py-2 focus-within:ring-2 focus-within:ring-ring transition-shadow">
          <textarea
            bind:this={inputEl}
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
            <Send size={14} />
          </button>
        </div>
        <p class="text-xs text-muted-foreground mt-1.5 text-center">
          Enter untuk kirim · Shift+Enter untuk baris baru
        </p>
      </div>
    </div>
  </div>
</div>