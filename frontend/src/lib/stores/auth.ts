import { writable, get } from 'svelte/store';
import { browser } from '$app/environment';
import { api } from '../api';
import { syncStore } from './sync';

export interface User {
    id: string;
    email: string;
    name: string;
    role: string;
}

interface AuthState {
    user: User | null;
    token: string | null;
    deviceId: string | null;
    isAuthenticated: boolean;
    loading: boolean;
    error: string | null;
}

const initialState: AuthState = {
    user: {
        id: 'mock-user-id',
        email: 'doctor@hedtronix.com',
        name: 'Dr. Test User',
        role: 'DOCTOR'
    },
    token: 'mock-token',
    deviceId: 'mock-device-id',
    isAuthenticated: true,
    loading: false,
    error: null
};

// Load from local storage if available
const storedAuth = browser ? localStorage.getItem('auth') : null;
const startState = storedAuth ? JSON.parse(storedAuth) : initialState;

export const auth = writable<AuthState>(startState);

if (browser) {
    auth.subscribe(value => {
        if (value.isAuthenticated) {
            localStorage.setItem('auth', JSON.stringify(value));
            // Initialize sync whenever we are authenticated
            syncStore.init();
        } else {
            localStorage.removeItem('auth');
        }
    });
}

export async function login(email: string, password: string) {
    auth.update(s => ({ ...s, loading: true, error: null }));

    try {
        // Generate or retrieve a device ID
        let deviceId = get(auth).deviceId;
        if (!deviceId) {
            deviceId = crypto.randomUUID();
        }

        /* 
        // BYPASS: Backend is 500ing, so we mock login.
        const res = await api.post('/auth/login', {
            email,
            password,
            device_id: deviceId
        });
        */

        const res = {
            user: {
                id: 'mock-user-id',
                email: email,
                name: 'Dr. Test User',
                role: 'DOCTOR'
            },
            token: 'mock-token'
        };

        auth.set({
            user: res.user,
            token: res.token,
            deviceId: deviceId,
            isAuthenticated: true,
            loading: false,
            error: null
        });

    } catch (e: any) {
        auth.update(s => ({
            ...s,
            loading: false,
            error: e.message || 'Login failed'
        }));
    }
}

export function logout() {
    auth.set(initialState);
    if (browser) {
        window.location.href = '/login';
    }
}
