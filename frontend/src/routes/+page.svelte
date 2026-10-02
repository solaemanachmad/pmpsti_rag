<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isLoggedIn } from '$lib/stores/auth';
  import { askPublic } from '$lib/api/client';
  import type { GuestAskResponse, SourceRef } from '$lib/api/client';
  import { Send, Loader2, MessageSquare, ChevronDown, FileText, LogIn, UserPlus } from 'lucide-svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  marked.use({
    renderer: {
      paragraph(token) {
        const text = (token.text || '').replace(/\[(\d+)\]/g, '<sup class="citation-ref">[$1]</sup>');
        return `<p>${text}</p>`;
      }
    }
  });

  function renderMarkdown(text: string): string {
    const raw = marked.parse(text) as string;
    return DOMPurify.sanitize(raw, { ADD_ATTR: ['target', 'rel'] });
  }

  const GUEST_QUOTA = 5;

  // Guest token — persistent via localStorage
  function getGuestToken(): string {
    try {
      let token = localStorage.getItem('guest_token');
      if (!token) {
        token = 'guest_' + Math.random().toString(36).slice(2) + Date.now().toString(36);
        localStorage.setItem('guest_token', token);
      }
      return token;
    } catch { return 'guest_' + Math.random().toString(36).slice(2); }
  }

  function getQuestionsUsed(): number {
    try { return parseInt(localStorage.getItem('guest_questions_used') ?? '0', 10); } catch { return 0; }
  }
  function setQuestionsUsed(n: number) {
    try { localStorage.setItem('guest_questions_used', String(n)); } catch {}
  }

  interface Message {
    role: 'user' | 'assistant';
    content: string;
    sources?: SourceRef[];
  }

  let messages: Message[] = [];
  let query = '';
  let streamingText = '';
  let isStreaming = false;
  let error = '';
  let quotaExceeded = false;
  let questionsUsed = 0;
  let questionsLeft = GUEST_QUOTA;
  let messagesEl: HTMLDivElement;
  let cancelFn: (() => void) | null = null;

  onMount(() => {
    // Kalau sudah login, redirect ke /chat
    if ($isLoggedIn) { goto('/chat'); return; }
    questionsUsed = getQuestionsUsed();
    questionsLeft = Math.max(GUEST_QUOTA - questionsUsed, 0);
    if (questionsLeft === 0) quotaExceeded = true;
  });

  async function scrollBottom() {
    await new Promise(r => setTimeout(r, 30));
    if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  async function send() {
    const q = query.trim();
    if (!q || isStreaming || quotaExceeded) return;
    query = '';
    error = '';

    messages = [...messages, { role: 'user', content: q }];
    await scrollBottom();

    isStreaming = true;
    streamingText = '';

    const token = getGuestToken();

    cancelFn = await askPublic(
      q,
      token,
      (chunk) => { streamingText += chunk; scrollBottom(); },
      (res: GuestAskResponse) => {
        messages = [...messages, {
          role: 'assistant',
          content: streamingText,
          sources: res.sources
        }];
        streamingText = '';
        isStreaming = false;
        questionsUsed = res.questions_used;
        questionsLeft = res.questions_left;
        setQuestionsUsed(questionsUsed);
        if (questionsLeft <= 0) quotaExceeded = true;
        scrollBottom();
      },
      (err: string, exceeded?: boolean) => {
        if (exceeded) {
          quotaExceeded = true;
          questionsLeft = 0;
        } else {
          error = err;
        }
        isStreaming = false;
        streamingText = '';
      }
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); send(); }
  }

  function autoResize(e: Event) {
    const el = e.target as HTMLTextAreaElement;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 140) + 'px';
  }
</script>

<svelte:head><title>PMPSTI RAG — Tanya Dokumen Akademik</title></svelte:head>

<div class="min-h-screen flex flex-col bg-background">

  <!-- Header -->
  <header class="border-b bg-card px-4 py-3 flex items-center justify-between">
    <div class="flex items-center gap-2.5">
      <div class="w-7 h-7 rounded-lg bg-primary flex items-center justify-center shrink-0">
        <MessageSquare size={14} class="text-primary-foreground" />
      </div>
      <span class="font-semibold text-sm">PMPSTI RAG</span>
    </div>
    <div class="flex items-center gap-2">
      <a href="/login"
        class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors px-2 py-1.5 rounded-md hover:bg-muted">
        <LogIn size={14} />
        <span class="hidden sm:inline">Masuk</span>
      </a>
      <a href="/register"
        class="flex items-center gap-1.5 text-sm bg-primary text-primary-foreground px-3 py-1.5 rounded-lg hover:opacity-90 transition-opacity font-medium">
        <UserPlus size={14} />
        <span>Daftar</span>
      </a>
    </div>
  </header>

  <!-- Chat area -->
  <div class="flex-1 flex flex-col max-w-2xl w-full mx-auto px-4 py-4 min-h-0">

    <!-- Empty state / intro -->
    {#if messages.length === 0 && !isStreaming}
      <div class="flex-1 flex flex-col items-center justify-center text-center py-12">
        <div class="w-14 h-14 rounded-2xl bg-muted flex items-center justify-center mb-5">
          <MessageSquare size={26} class="text-muted-foreground" />
        </div>
        <h1 class="text-2xl font-bold mb-2">Tanya Dokumen PMPSTI</h1>
        <p class="text-muted-foreground text-sm max-w-sm leading-relaxed">
          Ajukan pertanyaan tentang regulasi, kurikulum, atau panduan akademik.
          AI akan menjawab berdasarkan dokumen resmi.
        </p>
        {#if questionsLeft > 0}
          <p class="mt-4 text-xs text-muted-foreground bg-muted px-3 py-1.5 rounded-full">
            {questionsLeft} pertanyaan gratis tersisa · <a href="/register" class="underline hover:text-foreground">Daftar</a> untuk akses penuh
          </p>
        {/if}
      </div>
    {:else}
      <!-- Messages -->
      <div bind:this={messagesEl} class="flex-1 overflow-y-auto space-y-5 py-2 pr-1">
        {#each messages as msg (msg.content + msg.role)}
          {#if msg.role === 'user'}
            <div class="flex justify-end">
              <div class="bg-primary text-primary-foreground px-4 py-2.5 rounded-2xl rounded-tr-sm max-w-[80%] text-sm">
                {msg.content}
              </div>
            </div>
          {:else}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0 mt-0.5">
                <MessageSquare size={13} class="text-muted-foreground" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="text-sm leading-relaxed">
                  {@html renderMarkdown(msg.content)}
                </div>
                {#if msg.sources && msg.sources.length > 0}
                  <details class="mt-2.5 group">
                    <summary class="flex items-center gap-1.5 text-xs text-muted-foreground
                                   hover:text-foreground cursor-pointer select-none list-none w-fit">
                      <FileText size={11} />
                      <span>{msg.sources.length} sumber</span>
                      <ChevronDown size={11} class="transition-transform group-open:rotate-180" />
                    </summary>
                    <ol class="mt-2 space-y-1.5">
                      {#each msg.sources as src, i}
                        {@const hasUrl = src.source_url && src.source_url.startsWith('http')}
                        {@const domain = hasUrl ? (() => { try { return new URL(src.source_url).hostname.replace(/^www\./, '') } catch { return '' } })() : ''}
                        <li class="flex items-start gap-2 text-xs">
                          <span class="shrink-0 w-4 h-4 rounded-full bg-muted text-muted-foreground
                                       flex items-center justify-center text-[10px] font-medium mt-0.5">
                            {i + 1}
                          </span>
                          <div class="flex-1 min-w-0">
                            {#if hasUrl}
                              <a href={src.source_url} target="_blank" rel="noopener noreferrer"
                                 class="font-medium text-foreground hover:text-primary hover:underline
                                        line-clamp-1 leading-snug">
                                {src.title || domain}
                              </a>
                            {:else}
                              <span class="font-medium text-foreground line-clamp-1 leading-snug">
                                {src.title}
                              </span>
                            {/if}
                            <p class="text-muted-foreground line-clamp-1 leading-relaxed mt-0.5">
                              {src.snippet}
                            </p>
                            {#if domain}
                              <span class="text-[10px] text-muted-foreground/70">{domain}</span>
                            {/if}
                          </div>
                        </li>
                      {/each}
                    </ol>
                  </details>
                {/if}
              </div>
            </div>
          {/if}
        {/each}

        <!-- Streaming -->
        {#if isStreaming || streamingText}
          <div class="flex gap-3">
            <div class="w-7 h-7 rounded-full bg-muted flex items-center justify-center shrink-0 mt-0.5">
              {#if isStreaming && !streamingText}
                <Loader2 size={13} class="animate-spin text-muted-foreground" />
              {:else}
                <MessageSquare size={13} class="text-muted-foreground" />
              {/if}
            </div>
            <div class="text-sm leading-relaxed flex-1">
              {#if streamingText}
                {@html renderMarkdown(streamingText)}
                {#if isStreaming}<span class="cursor-blink"></span>{/if}
              {:else}
                <span class="text-muted-foreground text-xs">Sedang mencari...</span>
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
    {/if}

    <!-- Quota exceeded banner -->
    {#if quotaExceeded}
      <div class="mt-4 bg-muted border rounded-xl p-4 text-center">
        <p class="text-sm font-medium mb-1">Batas pertanyaan gratis ({GUEST_QUOTA}) habis</p>
        <p class="text-xs text-muted-foreground mb-3">Daftar akun gratis untuk pertanyaan tanpa batas dan simpan riwayat chat.</p>
        <div class="flex gap-2 justify-center">
          <a href="/register"
            class="bg-primary text-primary-foreground text-sm font-medium px-4 py-2 rounded-lg hover:opacity-90 transition-opacity">
            Daftar gratis
          </a>
          <a href="/login"
            class="border text-sm px-4 py-2 rounded-lg hover:bg-muted transition-colors">
            Masuk
          </a>
        </div>
      </div>
    {:else}
      <!-- Input -->
      <div class="mt-4 {messages.length > 0 ? '' : ''}">
        <!-- Counter -->
        {#if questionsUsed > 0 || messages.length > 0}
          <p class="text-xs text-muted-foreground text-center mb-2">
            {questionsLeft} pertanyaan gratis tersisa ·
            <a href="/register" class="underline hover:text-foreground">Daftar</a> untuk akses penuh
          </p>
        {/if}
        <div class="flex items-end gap-2 border rounded-xl bg-card px-3 py-2 focus-within:ring-2 focus-within:ring-ring transition-shadow">
          <textarea
            bind:value={query}
            on:keydown={handleKeydown}
            on:input={autoResize}
            placeholder="Tanya sesuatu tentang dokumen PMPSTI..."
            rows="1"
            disabled={isStreaming}
            class="flex-1 resize-none bg-transparent text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed min-h-[24px] max-h-36 py-0.5"
          ></textarea>
          <button
            on:click={send}
            disabled={!query.trim() || isStreaming}
            class="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg bg-primary text-primary-foreground disabled:opacity-40 hover:opacity-90 transition-opacity"
          >
            <Send size={14} />
          </button>
        </div>
      </div>
    {/if}

  </div>
</div>
