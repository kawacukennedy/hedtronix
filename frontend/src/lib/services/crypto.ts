
const ALGORITHM = 'AES-GCM';
const KEY_USAGE: KeyUsage[] = ['encrypt', 'decrypt'];
const KEY_LENGTH = 256;

export class CryptoService {
    private key: CryptoKey | null = null;
    private static instance: CryptoService;

    private constructor() { }

    static getInstance(): CryptoService {
        if (!CryptoService.instance) {
            CryptoService.instance = new CryptoService();
        }
        return CryptoService.instance;
    }

    /**
     * Initialize the crypto service with a master key.
     * In a real app, this would be derived from the user's password using PBKDF2/Argon2
     * or retrieved from a secure enclave. For this demo, we'll generate/store in session.
     */
    async init(passphrase?: string): Promise<void> {
        if (this.key) return;

        if (passphrase) {
            this.key = await this.deriveKey(passphrase);
        } else {
            // For prototype/demo without login flow active, try to load or generate a random one
            // Storing in localStorage is NOT secure for production, but allows persistence across reloads for the demo
            const storedKey = localStorage.getItem('hedtronix_master_key');
            if (storedKey) {
                this.key = await this.importKey(storedKey);
            } else {
                this.key = await window.crypto.subtle.generateKey(
                    { name: ALGORITHM, length: KEY_LENGTH },
                    true,
                    KEY_USAGE
                );
                const exported = await this.exportKey(this.key);
                localStorage.setItem('hedtronix_master_key', exported);
            }
        }
    }

    async encrypt(data: any): Promise<{ iv: string; data: string }> {
        if (!this.key) await this.init();
        if (!this.key) throw new Error('Crypto not initialized');

        const iv = window.crypto.getRandomValues(new Uint8Array(12));
        const encodedData = new TextEncoder().encode(JSON.stringify(data));

        const encrypted = await window.crypto.subtle.encrypt(
            { name: ALGORITHM, iv },
            this.key,
            encodedData
        );

        return {
            iv: this.arrayBufferToBase64(iv),
            data: this.arrayBufferToBase64(encrypted)
        };
    }

    async decrypt(encryptedData: string, ivFn: string): Promise<any> {
        if (!this.key) await this.init();
        if (!this.key) throw new Error('Crypto not initialized');

        const iv = this.base64ToArrayBuffer(ivFn);
        const data = this.base64ToArrayBuffer(encryptedData);

        try {
            const decrypted = await window.crypto.subtle.decrypt(
                { name: ALGORITHM, iv },
                this.key,
                data
            );
            return JSON.parse(new TextDecoder().decode(decrypted));
        } catch (e) {
            console.error('Decryption failed', e);
            throw new Error('Failed to decrypt data');
        }
    }

    // --- Helpers ---

    private async deriveKey(passphrase: string): Promise<CryptoKey> {
        const enc = new TextEncoder();
        const keyMaterial = await window.crypto.subtle.importKey(
            "raw",
            enc.encode(passphrase),
            { name: "PBKDF2" },
            false,
            ["deriveBits", "deriveKey"]
        );

        return window.crypto.subtle.deriveKey(
            {
                name: "PBKDF2",
                salt: enc.encode("hedtronix-salt"), // In prod, unique salt per user
                iterations: 100000,
                hash: "SHA-256"
            },
            keyMaterial,
            { name: ALGORITHM, length: KEY_LENGTH },
            true,
            KEY_USAGE
        );
    }

    private async exportKey(key: CryptoKey): Promise<string> {
        const exported = await window.crypto.subtle.exportKey("jwk", key);
        return JSON.stringify(exported);
    }

    private async importKey(jwkStr: string): Promise<CryptoKey> {
        const jwk = JSON.parse(jwkStr);
        return window.crypto.subtle.importKey(
            "jwk",
            jwk,
            { name: ALGORITHM },
            true,
            KEY_USAGE
        );
    }

    private arrayBufferToBase64(buffer: ArrayBuffer): string {
        let binary = '';
        const bytes = new Uint8Array(buffer);
        const len = bytes.byteLength;
        for (let i = 0; i < len; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return window.btoa(binary);
    }

    private base64ToArrayBuffer(base64: string): Uint8Array {
        const binary_string = window.atob(base64);
        const len = binary_string.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) {
            bytes[i] = binary_string.charCodeAt(i);
        }
        return bytes;
    }
}

export const cryptoService = CryptoService.getInstance();
