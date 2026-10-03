<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isLoggedIn } from '$lib/stores/auth';
  import { askPublic } from '$lib/api/client';
  import type { GuestAskResponse, SourceRef } from '$lib/api/client';
  import { Send, Loader2, ChevronDown, FileText, LogIn, UserPlus, ExternalLink } from 'lucide-svelte';
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

  function getGuestToken(): string {
    try {
      let t = localStorage.getItem('guest_token');
      if (!t) {
        t = 'guest_' + Math.random().toString(36).slice(2) + Date.now().toString(36);
        localStorage.setItem('guest_token', t);
      }
      return t;
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

  const suggestions = [
    'Apa syarat kelulusan program magister PMPSTI?',
    'Bagaimana mekanisme pembayaran UKT?',
    'Prosedur pengajuan cuti akademik mahasiswa',
  ];

  onMount(() => {
    if ($isLoggedIn) { goto('/chat'); return; }
    questionsUsed = getQuestionsUsed();
    questionsLeft = Math.max(GUEST_QUOTA - questionsUsed, 0);
    if (questionsLeft === 0) quotaExceeded = true;
  });

  async function scrollBottom() {
    await new Promise(r => setTimeout(r, 30));
    if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
  }

  async function send(q?: string) {
    const text = (q ?? query).trim();
    if (!text || isStreaming || quotaExceeded) return;
    query = '';
    error = '';
    messages = [...messages, { role: 'user', content: text }];
    await scrollBottom();
    isStreaming = true;
    streamingText = '';
    const token = getGuestToken();
    await askPublic(
      text, token,
      (chunk) => { streamingText += chunk; scrollBottom(); },
      (res: GuestAskResponse) => {
        messages = [...messages, { role: 'assistant', content: streamingText, sources: res.sources }];
        streamingText = '';
        isStreaming = false;
        questionsUsed = res.questions_used;
        questionsLeft = res.questions_left;
        setQuestionsUsed(questionsUsed);
        if (questionsLeft <= 0) quotaExceeded = true;
        scrollBottom();
      },
      (err: string, exceeded?: boolean) => {
        if (exceeded) { quotaExceeded = true; questionsLeft = 0; }
        else { error = err; }
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

  function sourceTitle(src: SourceRef): string {
    if (src.title) return src.title;
    try { return new URL(src.source_url).hostname.replace(/^www\./, ''); } catch { return 'Dokumen'; }
  }

  function sourceDomain(src: SourceRef): string {
    try { return new URL(src.source_url).hostname.replace(/^www\./, ''); } catch { return ''; }
  }
</script>

<svelte:head>
  <title>PMPSTI RAG — Asisten Akademik UGM</title>
  <meta name="description" content="Tanya dokumen akademik PMPSTI Universitas Gadjah Mada. Dijawab berdasarkan dokumen resmi.">
</svelte:head>

<div class="min-h-screen flex flex-col bg-background">

  <!-- ── Header ── -->
  <header class="border-b bg-[#002147] text-white px-4 py-3 flex items-center justify-between sticky top-0 z-30">
    <div class="flex items-center gap-3">
      <img src="/ugm-logo.svg" alt="Logo UGM" class="w-9 h-9 shrink-0" />
      <div class="leading-tight">
        <div class="font-semibold text-sm tracking-wide">PMPSTI RAG</div>
        <div class="text-[11px] text-white/60">Universitas Gadjah Mada</div>
      </div>
    </div>
    <nav class="flex items-center gap-1.5">
      <a href="/login"
        class="flex items-center gap-1.5 text-sm text-white/80 hover:text-white hover:bg-white/10
               px-3 py-1.5 rounded-lg transition-colors font-medium">
        <LogIn size={14} />
        <span class="hidden sm:inline">Masuk</span>
      </a>
      <a href="/register"
        class="flex items-center gap-1.5 text-sm font-semibold
               bg-[#F5A623] text-[#002147] px-3 py-1.5 rounded-lg
               hover:brightness-105 transition-all">
        <UserPlus size={14} />
        <span>Daftar</span>
      </a>
    </nav>
  </header>

  <!-- ── Chat area ── -->
  <div class="flex-1 flex flex-col max-w-2xl w-full mx-auto px-4 py-4 min-h-0">

    {#if messages.length === 0 && !isStreaming}
      <!-- ── Empty state ── -->
      <div class="flex-1 flex flex-col items-center justify-center text-center py-10">
        <div class="w-16 h-16 rounded-2xl bg-[#002147] flex items-center justify-center mb-5 shadow-lg">
          <img src="/ugm-logo.svg" alt="" class="w-10 h-10" />
        </div>
        <h1 class="text-2xl font-bold text-foreground mb-2 leading-tight">
          Asisten Dokumen Akademik
        </h1>
        <p class="text-muted-foreground text-sm max-w-xs leading-relaxed">
          Pertanyaan seputar regulasi, kurikulum, dan panduan akademik PMPSTI dijawab berdasarkan dokumen resmi.
        </p>

        <!-- Suggestion chips -->
        <div class="mt-6 flex flex-col gap-2 w-full max-w-sm">
          {#each suggestions as s}
            <button
              on:click={() => send(s)}
              class="text-left text-sm px-4 py-2.5 rounded-xl border bg-card
                     hover:border-primary/50 hover:bg-primary/5 transition-colors
                     text-muted-foreground hover:text-foreground">
              {s}
            </button>
          {/each}
        </div>

        {#if questionsLeft > 0}
          <p class="mt-5 text-xs text-muted-foreground">
            {questionsLeft} pertanyaan gratis ·
            <a href="/register" class="text-primary font-medium hover:underline">Daftar</a>
            untuk akses penuh
          </p>
        {/if}
      </div>

    {:else}
      <!-- ── Messages ── -->
      <div bind:this={messagesEl} class="flex-1 overflow-y-auto space-y-5 py-2 pr-1">
        {#each messages as msg}
          {#if msg.role === 'user'}
            <div class="flex justify-end">
              <div class="bg-[#0055A5] text-white px-4 py-2.5 rounded-2xl rounded-tr-sm
                          max-w-[80%] text-sm leading-relaxed">
                {msg.content}
              </div>
            </div>
          {:else}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-[#002147] flex items-center justify-center shrink-0 mt-0.5">
                <img src="/ugm-logo.svg" alt="" class="w-4 h-4" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="prose text-sm leading-relaxed">
                  {@html renderMarkdown(msg.content)}
                </div>

                <!-- ── Sources — kartu yang bisa diklik ── -->
                {#if msg.sources && msg.sources.length > 0}
                  <div class="mt-3">
                    <button
                      class="flex items-center gap-1.5 text-xs text-muted-foreground
                             hover:text-foreground mb-2 transition-colors group"
                      on:click={(e) => {
                        const el = e.currentTarget.closest('.source-section');
                        el?.classList.toggle('open');
                      }}
                    >
                      <FileText size={11} />
                      <span>{msg.sources.length} sumber dokumen</span>
                      <ChevronDown size={11} class="transition-transform group-[.open]:rotate-180 ml-0.5" />
                    </button>

                    <!-- sumber selalu tampil (no toggle JS needed) -->
                    <div class="grid gap-2">
                      {#each msg.sources as src, i}
                        {@const hasUrl = src.source_url?.startsWith('http')}
                        <svelte:element
                          this={hasUrl ? 'a' : 'div'}
                          href={hasUrl ? src.source_url : undefined}
                          target={hasUrl ? '_blank' : undefined}
                          rel={hasUrl ? 'noopener noreferrer' : undefined}
                          class="source-card group"
                        >
                          <!-- Nomor sumber -->
                          <span class="shrink-0 w-5 h-5 rounded-md bg-primary/10 text-primary
                                       flex items-center justify-center text-[10px] font-bold mt-0.5">
                            {i + 1}
                          </span>
                          <div class="flex-1 min-w-0">
                            <div class="flex items-start justify-between gap-2">
                              <p class="text-xs font-semibold text-foreground line-clamp-1 leading-snug">
                                {sourceTitle(src)}
                              </p>
                              {#if hasUrl}
                                <ExternalLink size={11} class="shrink-0 text-muted-foreground/50 group-hover:text-primary mt-0.5 transition-colors" />
                              {/if}
                            </div>
                            {#if src.snippet}
                              <p class="text-xs text-muted-foreground mt-0.5 line-clamp-2 leading-relaxed">
                                {src.snippet}
                              </p>
                            {/if}
                            {#if hasUrl}
                              <p class="text-[10px] text-primary/70 mt-1">{sourceDomain(src)}</p>
                            {/if}
                          </div>
                        </svelte:element>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>
          {/if}
        {/each}

        <!-- Streaming -->
        {#if isStreaming || streamingText}
          <div class="flex gap-3">
            <div class="w-7 h-7 rounded-full bg-[#002147] flex items-center justify-center shrink-0 mt-0.5">
              {#if isStreaming && !streamingText}
                <Loader2 size={13} class="animate-spin text-white/60" />
              {:else}
                <img src="/ugm-logo.svg" alt="" class="w-4 h-4" />
              {/if}
            </div>
            <div class="prose text-sm leading-relaxed flex-1">
              {#if streamingText}
                {@html renderMarkdown(streamingText)}
                {#if isStreaming}<span class="cursor-blink"></span>{/if}
              {:else}
                <span class="text-muted-foreground text-xs">Sedang mencari di dokumen...</span>
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

    <!-- ── Quota banner ── -->
    {#if quotaExceeded}
      <div class="mt-4 border rounded-xl p-5 text-center bg-card">
        <p class="text-sm font-semibold mb-1">Batas {GUEST_QUOTA} pertanyaan gratis tercapai</p>
        <p class="text-xs text-muted-foreground mb-4">
          Daftar dengan email <strong>@mail.ugm.ac.id</strong> untuk pertanyaan tak terbatas dan riwayat chat.
        </p>
        <div class="flex gap-2 justify-center">
          <a href="/register" class="btn-gold">Daftar gratis</a>
          <a href="/login" class="btn-ghost border">Sudah punya akun</a>
        </div>
      </div>

    {:else}
      <!-- ── Input ── -->
      <div class="mt-4">
        {#if questionsUsed > 0 || messages.length > 0}
          <p class="text-xs text-muted-foreground text-center mb-2">
            {questionsLeft} pertanyaan gratis tersisa ·
            <a href="/register" class="text-primary hover:underline font-medium">Daftar</a>
            untuk akses penuh
          </p>
        {/if}
        <div class="flex items-end gap-2 border rounded-xl bg-card px-3 py-2
                    focus-within:ring-2 focus-within:ring-ring focus-within:border-primary/40
                    transition-shadow">
          <textarea
            bind:value={query}
            on:keydown={handleKeydown}
            on:input={autoResize}
            placeholder="Tanya tentang akademik PMPSTI UGM..."
            rows="1"
            disabled={isStreaming}
            class="flex-1 resize-none bg-transparent text-sm outline-none
                   placeholder:text-muted-foreground disabled:cursor-not-allowed
                   min-h-[24px] max-h-36 py-0.5"
          ></textarea>
          <button
            on:click={() => send()}
            disabled={!query.trim() || isStreaming}
            class="shrink-0 w-8 h-8 flex items-center justify-center rounded-lg
                   bg-[#0055A5] text-white disabled:opacity-40
                   hover:bg-[#002147] transition-colors"
          >
            <Send size={14} />
          </button>
        </div>
        <p class="text-[10px] text-muted-foreground/60 text-center mt-2">
          Jawaban berdasarkan dokumen resmi PMPSTI UGM · Bukan pengganti konsultasi resmi
        </p>
      </div>
    {/if}

  </div>
</div>
