import { auth } from './stores/auth';
import { get } from 'svelte/store';

const API_BASE = '/api/v1';

class ApiClient {
    private async request(endpoint: string, options: RequestInit = {}) {
        const authState = get(auth);
        const headers = new Headers(options.headers);

        if (authState.token) {
            headers.set('Authorization', `Bearer ${authState.token}`);
        }

        if (!headers.has('Content-Type')) {
            headers.set('Content-Type', 'application/json');
        }

        const response = await fetch(`${API_BASE}${endpoint}`, {
            ...options,
            headers
        });

        if (response.status === 401) {
            // Token expired or invalid
            auth.update(s => ({ ...s, isAuthenticated: false, token: null, user: null }));
            if (window.location.pathname !== '/login') {
                window.location.href = '/login';
            }
            throw new Error('Unauthorized');
        }

        if (!response.ok) {
            const error = await response.json().catch(() => ({ message: 'Unknown error' }));
            throw new Error(error.message || `API Error: ${response.statusText}`);
        }

        return response.json();
    }

    get(endpoint: string) {
        return this.request(endpoint, { method: 'GET' });
    }

    post(endpoint: string, data: any) {
        return this.request(endpoint, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    }

    put(endpoint: string, data: any) {
        return this.request(endpoint, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    }

    delete(endpoint: string) {
        return this.request(endpoint, { method: 'DELETE' });
    }
}

export const api = new ApiClient();
