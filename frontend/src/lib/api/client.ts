const BASE = '/api';

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem('token');
}

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> ?? {})
  };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  const res = await fetch(`${BASE}${path}`, { ...options, headers });
  const json = await res.json();
  if (!res.ok) throw new Error(json?.error ?? `HTTP ${res.status}`);
  return (json.data ?? json) as T;
}

// ── Auth ──
export const auth = {
  register: (email: string, password: string, display_name?: string) =>
    request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, display_name })
    }),
  login: (email: string, password: string) =>
    request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ email, password })
    }),
  me: () => request<UserPublic>('/auth/me'),
  updateProfile: (data: { display_name?: string; email?: string }) =>
    request('/auth/me', { method: 'PATCH', body: JSON.stringify(data) }),
  updatePassword: (current_password: string, new_password: string) =>
    request('/auth/me/password', {
      method: 'PATCH',
      body: JSON.stringify({ current_password, new_password })
    })
};

// ── Chat ──
export const chat = {
  ask: (query: string, session_id?: string, category_filter?: string, top_k = 5) =>
    request<AskResponse>('/ask', {
      method: 'POST',
      body: JSON.stringify({ query, session_id, category_filter, top_k, search_mode: 'hybrid' })
    }),
  sessions: () => request<SessionListItem[]>('/sessions'),
  getSession: (id: string) => request<ChatSession>(`/sessions/${id}`),
  deleteSession: (id: string) => request(`/sessions/${id}`, { method: 'DELETE' }),
  renameSession: (id: string, title: string) =>
    request(`/sessions/${id}/title`, { method: 'PATCH', body: JSON.stringify({ title }) }),
  categories: () => request<string[]>('/categories')
};

// ── API Keys ──
export const keys = {
  list: () => request<ApiKeyInfo[]>('/keys'),
  create: (name: string, permissions?: string[], rate_limit?: number, expires_at?: string) =>
    request<CreateApiKeyResponse>('/keys', {
      method: 'POST',
      body: JSON.stringify({ name, permissions, rate_limit, expires_at })
    }),
  revoke: (id: number) => request(`/keys/${id}`, { method: 'DELETE' })
};

// ── Admin ──
export const admin = {
  stats: () => request<QueryLogStats>('/admin/stats')
};

// ── Streaming ask (word-by-word simulation karena backend belum SSE) ──
export function askStream(
  query: string,
  session_id: string | undefined,
  onChunk: (text: string) => void,
  onDone: (sources: SourceRef[], session_id: string, ms: number) => void,
  onError: (err: string) => void
): () => void {
  const token = getToken();
  const ctrl = new AbortController();
  let cancelled = false;

  (async () => {
    try {
      const res = await fetch(`${BASE}/ask`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {})
        },
        body: JSON.stringify({ query, session_id, search_mode: 'hybrid', top_k: 5 }),
        signal: ctrl.signal
      });

      if (!res.ok) {
        const j = await res.json().catch(() => ({}));
        onError(j?.error ?? `HTTP ${res.status}`);
        return;
      }

      const json = await res.json();
      // Backend Actix returns { success: true, data: { answer, session_id, sources, ... } }
      const data: AskResponse = json?.data ?? json;
      if (cancelled) return;

      const answer = data?.answer ?? '';
      const sid = data?.session_id ?? '';
      const srcs = data?.sources ?? [];
      const ms = data?.search_time_ms ?? 0;

      if (!answer) {
        onError('Jawaban kosong dari server');
        return;
      }

      // Stream word-by-word
      const words = answer.split(' ');
      for (let i = 0; i < words.length; i++) {
        if (cancelled) return;
        await new Promise(r => setTimeout(r, 25));
        onChunk((i === 0 ? '' : ' ') + words[i]);
      }
      onDone(srcs, sid, ms);
    } catch (e: any) {
      if (e.name !== 'AbortError' && !cancelled) onError(String(e));
    }
  })();

  return () => { cancelled = true; ctrl.abort(); };
}

// ── Types ──
export interface AuthResponse {
  token: string;
  token_type: string;
  expires_in: number;
  user: UserPublic;
}
export interface UserPublic {
  id: number;
  email: string;
  display_name: string;
  role: string;
  created_at: string;
}
export interface AskResponse {
  answer: string;
  session_id: string;
  sources: SourceRef[];
  search_time_ms: number;
}
export interface SourceRef {
  title: string;
  snippet: string;
  source_url: string;
  category: string;
  score: number;
}
export interface SessionListItem {
  id: string;
  title: string;
  updated_at: string;
  message_count: number;
}
export interface ChatSession {
  id: string;
  user_id: number;
  title: string;
  messages: ChatMessage[];
  created_at: string;
  updated_at: string;
}
export interface ChatSource {
  title: string;
  snippet: string;
  source_url: string;
  category: string;
  score: number;
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  created_at: string;
  sources?: ChatSource[];
}
export interface ApiKeyInfo {
  id: number;
  key_prefix: string;
  name: string;
  permissions: string[];
  rate_limit: number;
  is_active: boolean;
  last_used_at: string | null;
  created_at: string;
  expires_at: string | null;
}
export interface CreateApiKeyResponse {
  id: number;
  key: string;
  key_prefix: string;
  name: string;
}
export interface QueryLogStats {
  total_queries: number;
  unique_queries: number;
  avg_results: number;
  avg_search_time_ms: number;
  zero_result_queries: number;
  language_distribution: [string, number][];
  domain_distribution: [string, number][];
  queries_per_day: [string, number][];
  top_queries: [string, number][];
}