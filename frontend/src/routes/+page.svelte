<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import ThemeToggle from '$lib/components/ThemeToggle.svelte';
  import { isLoggedIn } from '$lib/stores/auth';
  import { askPublic, faq } from '$lib/api/client';
  import type { GuestAskResponse, SourceRef } from '$lib/api/client';
  import { Send, Loader2, FileText, LogIn, UserPlus, ExternalLink, Sparkles, ChevronDown } from 'lucide-svelte';
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

  let dark = false;
  let messages: Message[] = [];
  let query = '';
  let streamingText = '';
  let isStreaming = false;
  let error = '';
  let quotaExceeded = false;
  let questionsUsed = 0;
  let questionsLeft = GUEST_QUOTA;
  let messagesEl: HTMLDivElement;

  // FAQ — diisi dari API, fallback ke hardcoded jika gagal
  let faqQuestions: string[] = [];
  let faqOpen: number | null = null;       // index accordion yang sedang terbuka
  let faqAnswers: Record<number, string> = {}; // cache jawaban accordion
  let faqLoading: Record<number, boolean> = {};
  const FAQ_INITIAL = 3;                   // tampilkan 3 dulu
  let faqExpanded = false;                 // tombol "Lihat semua"

  const fallbackSuggestions = [
    'Apa syarat kelulusan program magister PMPSTI?',
    'Bagaimana mekanisme pembayaran UKT?',
    'Prosedur pengajuan cuti akademik mahasiswa',
  ];

  onMount(async () => {
    if ($isLoggedIn) { goto('/chat'); return; }
    dark = document.documentElement.classList.contains('dark');
    questionsUsed = getQuestionsUsed();
    questionsLeft = Math.max(GUEST_QUOTA - questionsUsed, 0);
    if (questionsLeft === 0) quotaExceeded = true;

    // Load FAQ dari backend
    try {
      const qs = await faq.list();
      faqQuestions = qs.length > 0 ? qs : fallbackSuggestions;
    } catch {
      faqQuestions = fallbackSuggestions;
    }

    // Sync dark state saat ThemeToggle mengubah kelas
    const observer = new MutationObserver(() => {
      dark = document.documentElement.classList.contains('dark');
    });
    observer.observe(document.documentElement, { attributeFilter: ['class'] });
    return () => observer.disconnect();
  });

  // Accordion: load jawaban via guest RAG
  async function toggleFaq(idx: number) {
    if (faqOpen === idx) { faqOpen = null; return; }
    faqOpen = idx;
    if (faqAnswers[idx] !== undefined) return; // sudah ada cache

    const q = faqQuestions[idx];
    faqLoading = { ...faqLoading, [idx]: true };
    let ans = '';
    const token = getGuestToken();
    await askPublic(
      q, token,
      (chunk) => { ans += chunk; faqAnswers = { ...faqAnswers, [idx]: ans }; },
      () => { faqLoading = { ...faqLoading, [idx]: false }; },
      () => { faqLoading = { ...faqLoading, [idx]: false }; faqAnswers = { ...faqAnswers, [idx]: 'Gagal memuat jawaban.' }; }
    );
  }

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
  <title>DTETI Menjawab — Asisten Akademik PMPSTI UGM</title>
  <meta name="description" content="Tanya dokumen akademik PMPSTI Universitas Gadjah Mada. AI menjawab berdasarkan dokumen resmi, instan, 24/7.">
</svelte:head>

<div class="min-h-screen flex flex-col bg-background">

  <!-- ── Header ── -->
  <!-- Light: #E7ECF3 (ugm-navy-100) | Dark: #14213D (ugm-ink) — sama dengan header authenticated -->
  <header class="border-b px-4 py-3 flex items-center justify-between sticky top-0 z-30 transition-colors duration-200
    {dark ? 'bg-[#14213D] text-[#EDEDED] border-[#3A4B66]' : 'bg-[#E7ECF3] text-[#0B2545] border-[#C5D0DE]'}">
    <div class="flex items-center gap-3">
      <img
        src={dark ? '/ugm-logo-white.png' : '/ugm-logo-navy.png'}
        alt="Logo UGM"
        class="h-9 w-auto shrink-0"
        onerror="this.src='/ugm-logo-white.png'"
      />
      <div class="leading-tight">
        <div class="font-bold text-base tracking-wide">DTETI</div>
        <div class="text-[11px] {dark ? 'text-[#EDEDED]/60' : 'text-[#0B2545]/60'}">Teknik Elektro & Teknologi Informasi UGM</div>
      </div>
    </div>
    <nav class="flex items-center gap-1.5">
      <ThemeToggle onDark={dark} />
      <a href="/login"
        class="flex items-center gap-1.5 text-sm px-3 py-1.5 rounded-lg transition-colors font-medium
          {dark ? 'text-[#EDEDED]/80 hover:text-[#EDEDED] hover:bg-white/10' : 'text-[#0B2545]/70 hover:text-[#0B2545] hover:bg-[#0B2545]/8'}">
        <LogIn size={14} />
        <span class="hidden sm:inline">Masuk</span>
      </a>
      <a href="/register"
        class="flex items-center gap-1.5 text-sm font-semibold
               bg-[#C99A2E] text-white px-3 py-1.5 rounded-lg
               hover:brightness-105 transition-all dark:bg-[#E3B54F] dark:text-[#0B2545]">
        <UserPlus size={14} />
        <span>Daftar</span>
      </a>
    </nav>
  </header>

  <!-- ── Chat area ── -->
  <div class="flex-1 flex flex-col max-w-2xl w-full mx-auto px-4 py-4 min-h-0">

    {#if messages.length === 0 && !isStreaming}
      <!-- ── Hero / Empty state ── -->
      <div class="flex-1 flex flex-col items-center justify-center text-center py-8 gap-0">

        <!-- Vector illustration -->
        <div class="w-full max-w-xs mx-auto mb-6 select-none" aria-hidden="true">
          <svg viewBox="0 0 320 200" fill="none" xmlns="http://www.w3.org/2000/svg" class="w-full h-auto">
            <!-- Background blobs -->
            <ellipse cx="160" cy="110" rx="130" ry="70" fill="#EEF4FF" class="dark-fill-navy"/>
            <ellipse cx="90" cy="85" rx="36" ry="36" fill="#DBEAFE" class="dark-fill-navy2"/>
            <ellipse cx="230" cy="130" rx="28" ry="28" fill="#E0F2FE" class="dark-fill-navy2"/>

            <!-- Laptop / screen -->
            <rect x="90" y="65" width="140" height="88" rx="8" fill="#002147"/>
            <rect x="96" y="71" width="128" height="76" rx="5" fill="#0A1628"/>
            <!-- screen glow -->
            <rect x="96" y="71" width="128" height="76" rx="5" fill="url(#screenGrad)" opacity="0.7"/>
            <!-- chat bubbles on screen -->
            <rect x="104" y="80" width="55" height="10" rx="5" fill="#0055A5" opacity="0.8"/>
            <rect x="166" y="80" width="38" height="10" rx="5" fill="#1E3A5F" opacity="0.6"/>
            <rect x="104" y="96" width="72" height="10" rx="5" fill="#1E3A5F" opacity="0.6"/>
            <rect x="183" y="96" width="33" height="10" rx="5" fill="#0055A5" opacity="0.8"/>
            <rect x="104" y="112" width="48" height="10" rx="5" fill="#0055A5" opacity="0.8"/>
            <!-- cursor blink -->
            <rect x="156" y="113" width="3" height="8" rx="1.5" fill="#F5A623">
              <animate attributeName="opacity" values="1;0;1" dur="1s" repeatCount="indefinite"/>
            </rect>

            <!-- Laptop base -->
            <path d="M78 153h164l-8-8H86L78 153Z" fill="#002147" opacity="0.9"/>
            <rect x="78" y="153" width="164" height="6" rx="3" fill="#001530"/>
            <!-- Laptop bottom pad -->
            <ellipse cx="160" cy="156" rx="82" ry="4" fill="#001530" opacity="0.5"/>

            <!-- Floating doc cards -->
            <g transform="translate(34,72)">
              <rect width="44" height="52" rx="6" fill="white" stroke="#E2E8F0" stroke-width="1"/>
              <rect x="6" y="8" width="32" height="4" rx="2" fill="#002147" opacity="0.7"/>
              <rect x="6" y="16" width="28" height="3" rx="1.5" fill="#CBD5E1"/>
              <rect x="6" y="23" width="30" height="3" rx="1.5" fill="#CBD5E1"/>
              <rect x="6" y="30" width="22" height="3" rx="1.5" fill="#CBD5E1"/>
              <rect x="6" y="40" width="32" height="6" rx="3" fill="#0055A5" opacity="0.15"/>
              <rect x="10" y="41" width="20" height="4" rx="2" fill="#0055A5" opacity="0.7"/>
              <animateTransform attributeName="transform" type="translate" values="34,72;34,68;34,72" dur="3s" repeatCount="indefinite"/>
            </g>
            <g transform="translate(244,88)">
              <rect width="38" height="44" rx="5" fill="white" stroke="#E2E8F0" stroke-width="1"/>
              <rect x="5" y="7" width="28" height="3.5" rx="1.75" fill="#002147" opacity="0.7"/>
              <rect x="5" y="15" width="24" height="3" rx="1.5" fill="#CBD5E1"/>
              <rect x="5" y="22" width="26" height="3" rx="1.5" fill="#CBD5E1"/>
              <rect x="5" y="32" width="28" height="6" rx="3" fill="#F5A623" opacity="0.2"/>
              <rect x="9" y="33" width="17" height="4" rx="2" fill="#F5A623" opacity="0.8"/>
              <animateTransform attributeName="transform" type="translate" values="244,88;244,84;244,88" dur="3.5s" repeatCount="indefinite"/>
            </g>

            <!-- Stars / sparkles -->
            <circle cx="68" cy="52" r="3" fill="#F5A623" opacity="0.8">
              <animate attributeName="r" values="3;5;3" dur="2s" repeatCount="indefinite"/>
            </circle>
            <circle cx="258" cy="62" r="2" fill="#0055A5" opacity="0.7">
              <animate attributeName="r" values="2;4;2" dur="2.5s" repeatCount="indefinite"/>
            </circle>
            <circle cx="42" cy="145" r="2.5" fill="#F5A623" opacity="0.6">
              <animate attributeName="r" values="2.5;4;2.5" dur="3s" repeatCount="indefinite"/>
            </circle>

            <defs>
              <linearGradient id="screenGrad" x1="96" y1="71" x2="224" y2="147" gradientUnits="userSpaceOnUse">
                <stop offset="0%" stop-color="#0055A5" stop-opacity="0.3"/>
                <stop offset="100%" stop-color="#002147" stop-opacity="0.1"/>
              </linearGradient>
            </defs>
          </svg>
        </div>

        <!-- Headline -->
        <div class="mb-1">
          <span class="inline-flex items-center gap-1.5 text-xs font-semibold tracking-widest
                       uppercase text-[#0055A5] bg-[#EEF4FF] px-3 py-1 rounded-full mb-3">
            <Sparkles size={11} />
            Dokumen Resmi
          </span>
        </div>
        <h1 class="text-3xl sm:text-4xl font-extrabold text-foreground mb-3 leading-tight tracking-tight">
          DTETI&nbsp;<span class="text-[#0055A5]">Menjawab</span>
        </h1>
        <p class="text-muted-foreground text-sm sm:text-base max-w-sm leading-relaxed">
          Punya pertanyaan soal akademik PMPSTI?<br>
          Tanya di sini — dijawab langsung dari dokumen resmi.
        </p>

        <!-- FAQ section -->
        {#if faqQuestions.length > 0}
          {@const visibleFaq = faqExpanded ? faqQuestions : faqQuestions.slice(0, FAQ_INITIAL)}
          <div class="mt-6 w-full max-w-lg">
            <p class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3 text-center">
              Pertanyaan Populer
            </p>
            <div class="flex flex-col gap-1.5">
              {#each visibleFaq as q, idx}
                <div class="border rounded-xl bg-card overflow-hidden transition-all">
                  <button
                    on:click={() => toggleFaq(idx)}
                    class="w-full text-left flex items-center justify-between gap-3
                           px-4 py-3 hover:bg-muted/40 transition-colors group">
                    <span class="text-sm text-foreground/90 group-hover:text-foreground leading-snug">
                      {q}
                    </span>
                    <span class="shrink-0 flex items-center gap-2">
                      <span
                        role="button"
                        tabindex="0"
                        on:click|stopPropagation={() => send(q)}
                        on:keydown|stopPropagation={(e) => e.key === 'Enter' && send(q)}
                        class="hidden sm:inline-flex items-center text-[10px] font-semibold px-2 py-0.5
                               bg-[#0055A5]/10 text-[#0055A5] rounded-full
                               hover:bg-[#0055A5] hover:text-white transition-colors cursor-pointer">
                        Tanya
                      </span>
                      <ChevronDown size={14}
                        class="text-muted-foreground transition-transform duration-200
                               {faqOpen === idx ? 'rotate-180' : ''}" />
                    </span>
                  </button>

                  {#if faqOpen === idx}
                    <div class="px-4 pb-3 border-t border-border/50">
                      {#if faqLoading[idx]}
                        <div class="flex items-center gap-2 text-xs text-muted-foreground pt-3">
                          <Loader2 size={12} class="animate-spin" /> Mencari jawaban…
                        </div>
                      {:else if faqAnswers[idx]}
                        <div class="prose prose-sm text-sm leading-relaxed pt-3 max-h-48 overflow-y-auto">
                          {@html renderMarkdown(faqAnswers[idx])}
                        </div>
                        <button
                          on:click={() => send(q)}
                          class="mt-3 text-xs text-[#0055A5] hover:underline font-medium">
                          Tanya lebih lanjut →
                        </button>
                      {:else}
                        <div class="flex items-center gap-2 text-xs text-muted-foreground pt-3">
                          <Loader2 size={12} class="animate-spin" /> Memuat…
                        </div>
                      {/if}
                    </div>
                  {/if}
                </div>
              {/each}
            </div>

            <!-- Tombol expand jika ada lebih dari FAQ_INITIAL -->
            {#if faqQuestions.length > FAQ_INITIAL}
              <button
                on:click={() => { faqExpanded = !faqExpanded; if (!faqExpanded) faqOpen = null; }}
                class="mt-2 w-full flex items-center justify-center gap-1.5 text-xs
                       text-muted-foreground hover:text-foreground transition-colors py-1.5">
                <ChevronDown size={13}
                  class="transition-transform duration-200 {faqExpanded ? 'rotate-180' : ''}" />
                {faqExpanded
                  ? 'Sembunyikan'
                  : `Lihat ${faqQuestions.length - FAQ_INITIAL} pertanyaan lainnya`}
              </button>
            {/if}
          </div>
        {/if}

        {#if questionsLeft > 0}
          <p class="mt-5 text-xs text-muted-foreground">
            {questionsLeft} pertanyaan gratis ·
            <a href="/register" class="text-[#0055A5] font-semibold hover:underline">Daftar</a>
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
              <div class="w-7 h-7 rounded-full bg-[#002147] flex items-center justify-center shrink-0 mt-0.5 p-1">
                <img src="/ugm-logo-white.png" alt="" class="w-full h-full object-contain" />
              </div>
              <div class="flex-1 min-w-0">
                <div class="prose text-sm leading-relaxed">
                  {@html renderMarkdown(msg.content)}
                </div>

                <!-- ── Source reference cards ── -->
                {#if msg.sources && msg.sources.length > 0}
                  <div class="mt-3">
                    <p class="flex items-center gap-1.5 text-xs text-muted-foreground mb-2">
                      <FileText size={11} />
                      <span>{msg.sources.length} sumber dokumen</span>
                    </p>
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
                          <span class="shrink-0 w-5 h-5 rounded-md bg-[#0055A5]/10 text-[#0055A5]
                                       flex items-center justify-center text-[10px] font-bold mt-0.5">
                            {i + 1}
                          </span>
                          <div class="flex-1 min-w-0">
                            <div class="flex items-start justify-between gap-2">
                              <p class="text-xs font-semibold text-foreground line-clamp-2 leading-snug">
                                {sourceTitle(src)}
                              </p>
                              {#if hasUrl}
                                <ExternalLink size={11} class="shrink-0 text-muted-foreground/50 group-hover:text-[#0055A5] mt-0.5 transition-colors" />
                              {/if}
                            </div>
                            {#if src.snippet}
                              <p class="text-xs text-muted-foreground mt-0.5 line-clamp-2 leading-relaxed">
                                {src.snippet}
                              </p>
                            {/if}
                            {#if src.category}
                              <p class="text-[10px] text-[#0055A5]/70 mt-1">{src.category}</p>
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
            <div class="w-7 h-7 rounded-full bg-[#002147] flex items-center justify-center shrink-0 mt-0.5 p-1">
              {#if isStreaming && !streamingText}
                <Loader2 size={13} class="animate-spin text-white/60" />
              {:else}
                <img src="/ugm-logo-white.png" alt="" class="w-full h-full object-contain" />
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
        <p class="text-sm font-semibold mb-1">Batas {GUEST_QUOTA} pertanyaan gratis tercapai 🎓</p>
        <p class="text-xs text-muted-foreground mb-4">
          Daftar dengan email <strong>@mail.ugm.ac.id</strong> untuk pertanyaan tak terbatas dan riwayat chat.
        </p>
        <div class="flex gap-2 justify-center">
          <a href="/register" class="btn-gold">Daftar gratis</a>
          <a href="/login" class="btn-ghost border">Sudah punya akun</a>
        </div>
      </div>

    {:else}
      <!-- ── Input area ── -->
      <div class="mt-4">
        {#if questionsUsed > 0 || messages.length > 0}
          <p class="text-xs text-muted-foreground text-center mb-2">
            {questionsLeft} pertanyaan gratis tersisa ·
            <a href="/register" class="text-[#0055A5] hover:underline font-medium">Daftar</a>
            untuk akses penuh
          </p>
        {/if}
        <div class="flex items-end gap-2 border rounded-xl bg-card px-3 py-2
                    focus-within:ring-2 focus-within:ring-[#0055A5]/40 focus-within:border-[#0055A5]/40
                    transition-shadow">
          <textarea
            bind:value={query}
            on:keydown={handleKeydown}
            on:input={autoResize}
            placeholder="Tanya sesuatu tentang akademik PMPSTI…"
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

<style>
  /* Dark mode tint for illustration blobs */
  :global(.dark) .dark-fill-navy { fill: #0A1628; }
  :global(.dark) .dark-fill-navy2 { fill: #0D1E3A; }
</style>
