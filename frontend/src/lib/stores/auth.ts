import { writable, derived } from 'svelte/store';
import type { UserPublic } from '$lib/api/client';

interface AuthState {
  user: UserPublic | null;
  ready: boolean;
}

function createAuthStore() {
  // Hydrate user dari localStorage (tidak sensitif — bukan token)
  const initial: AuthState =
    typeof localStorage !== 'undefined'
      ? {
          user: JSON.parse(localStorage.getItem('user') ?? 'null'),
          ready: false,
        }
      : { user: null, ready: false };

  const { subscribe, set, update } = writable<AuthState>(initial);

  return {
    subscribe,
    login(user: UserPublic) {
      // Token disimpan di httpOnly cookie oleh backend — tidak perlu di sini
      localStorage.setItem('user', JSON.stringify(user));
      set({ user, ready: true });
    },
    logout() {
      localStorage.removeItem('user');
      set({ user: null, ready: true });
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
// isLoggedIn: ada user di store (setelah auth.me() confirm dari cookie)
export const isLoggedIn = derived(authStore, $s => !!$s.user);
export const currentUser = derived(authStore, $s => $s.user);
export const authReady = derived(authStore, $s => $s.ready);
