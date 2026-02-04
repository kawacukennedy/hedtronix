import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface User {
    id: string;
    email: string;
    name: string;
    role: string;
}

interface AuthState {
    user: User | null;
    token: string | null;
    isAuthenticated: boolean;
}

const initialState: AuthState = {
    user: null,
    token: null,
    isAuthenticated: false
};

// Load from local storage if available
const storedAuth = browser ? localStorage.getItem('auth') : null;
const startState = storedAuth ? JSON.parse(storedAuth) : initialState;

export const auth = writable<AuthState>(startState);

if (browser) {
    auth.subscribe(value => {
        if (value.isAuthenticated) {
            localStorage.setItem('auth', JSON.stringify(value));
        } else {
            localStorage.removeItem('auth');
        }
    });
}

export function logout() {
    auth.set(initialState);
}
