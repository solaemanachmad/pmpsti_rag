import { writable, derived } from 'svelte/store';
import type { UserPublic } from '$lib/api/client';

interface AuthState {
  token: string | null;
  user: UserPublic | null;
  ready: boolean;  // true setelah auth.me() selesai (berhasil/gagal)
}

function createAuthStore() {
  // Hydrate dari localStorage saat pertama load
  const initial: AuthState =
    typeof localStorage !== 'undefined'
      ? {
          token: localStorage.getItem('token'),
          user: JSON.parse(localStorage.getItem('user') ?? 'null'),
          ready: false,
        }
      : { token: null, user: null, ready: false };

  const { subscribe, set, update } = writable<AuthState>(initial);

  return {
    subscribe,
    login(token: string, user: UserPublic) {
      localStorage.setItem('token', token);
      localStorage.setItem('user', JSON.stringify(user));
      set({ token, user, ready: true });
    },
    logout() {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      set({ token: null, user: null, ready: true });
    },
    setUser(user: UserPublic) {
      localStorage.setItem('user', JSON.stringify(user));
      update(s => ({ ...s, user, ready: true }));
    },
    setReady() {
      update(s => ({ ...s, ready: true }));
    }
  };
}

export const authStore = createAuthStore();
export const isLoggedIn = derived(authStore, $s => !!$s.token);
export const currentUser = derived(authStore, $s => $s.user);
export const authReady = derived(authStore, $s => $s.ready);
