import { ref } from 'vue';
import type { AdminAuthVerifyResponse } from '../types';

const STORAGE_KEY = 'kairos_admin_key';

export const adminKey = ref<string>(localStorage.getItem(STORAGE_KEY) || '');
export const isAuthenticated = ref<boolean>(true);
export const isAuthRequired = ref<boolean>(false);
export const showAuthModal = ref<boolean>(false);

export const authService = {
  getAdminKey(): string {
    return adminKey.value;
  },

  setAdminKey(key: string) {
    const trimmed = key.trim();
    adminKey.value = trimmed;
    if (trimmed) {
      localStorage.setItem(STORAGE_KEY, trimmed);
    } else {
      localStorage.removeItem(STORAGE_KEY);
    }
  },

  clearAdminKey() {
    adminKey.value = '';
    localStorage.removeItem(STORAGE_KEY);
  },

  getAuthHeaders(existingHeaders: HeadersInit = {}): HeadersInit {
    const headers = new Headers(existingHeaders);
    if (adminKey.value) {
      headers.set('X-Admin-Key', adminKey.value);
    }
    return headers;
  },

  handleUnauthorized() {
    isAuthenticated.value = false;
    showAuthModal.value = true;
  },

  async verify(testKey?: string): Promise<AdminAuthVerifyResponse> {
    const keyToTest = testKey !== undefined ? testKey.trim() : adminKey.value;
    const headers = new Headers();
    if (keyToTest) {
      headers.set('X-Admin-Key', keyToTest);
    }

    try {
      const res = await fetch('/api/admin/verify', { headers });
      if (res.status === 401) {
        isAuthenticated.value = false;
        isAuthRequired.value = true;
        showAuthModal.value = true;
        return { success: false, auth_required: true, authenticated: false };
      }

      if (res.ok) {
        const data: AdminAuthVerifyResponse = await res.json();
        isAuthenticated.value = true;
        isAuthRequired.value = data.auth_required;
        if (testKey !== undefined) {
          this.setAdminKey(keyToTest);
        }
        showAuthModal.value = false;
        return data;
      }

      return { success: false, auth_required: false, authenticated: false };
    } catch {
      // In case of network errors or server booting up
      return { success: false, auth_required: false, authenticated: true };
    }
  },
};
