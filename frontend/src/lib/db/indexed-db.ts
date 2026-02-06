
import { openDB, type DBSchema, type IDBPDatabase } from 'idb';
import { cryptoService } from '$lib/services/crypto';

// Encrypted wrapper type
interface EncryptedData {
    id: string;
    _encrypted: string;
    _iv: string;
    // We keep some indexable fields unencrypted for queries
    [key: string]: any;
}

interface HedtronixDB extends DBSchema {
    patients: {
        key: string;
        value: EncryptedData; // Encrypted
        indexes: { 'by-name': string };
    };
    appointments: {
        key: string;
        value: any; // Not encrypting appointments for this demo as they need complex querying
        indexes: { 'by-date': string; 'by-patient': string };
    };
    notes: {
        key: string;
        value: EncryptedData; // Encrypted
        indexes: { 'by-patient': string };
    };
    billing: {
        key: string;
        value: any;
        indexes: { 'by-patient': string; 'by-status': string };
    };
    sync_queue: {
        key: string;
        value: {
            id: string;
            entityType: string;
            entityId: string;
            operation: 'CREATE' | 'UPDATE' | 'DELETE';
            data: any;
            timestamp: number;
        };
    };
}

class Database {
    private dbName = 'hedtronix-db';
    private version = 1;
    private db: IDBPDatabase<HedtronixDB> | null = null;
    private initialized = false;

    async init() {
        if (this.initialized) return;

        // Ensure crypto is ready
        await cryptoService.init();

        this.db = await openDB<HedtronixDB>(this.dbName, this.version, {
            upgrade(db) {
                // Patients
                const patientStore = db.createObjectStore('patients', { keyPath: 'id' });
                patientStore.createIndex('by-name', 'lastName');

                // Appointments
                const aptStore = db.createObjectStore('appointments', { keyPath: 'id' });
                aptStore.createIndex('by-date', 'startTime');
                aptStore.createIndex('by-patient', 'patientId');

                // Notes
                const noteStore = db.createObjectStore('notes', { keyPath: 'id' });
                noteStore.createIndex('by-patient', 'patientId');

                // Billing
                const billingStore = db.createObjectStore('billing', { keyPath: 'id' });
                billingStore.createIndex('by-patient', 'patientId');
                billingStore.createIndex('by-status', 'status');

                // Sync Queue
                db.createObjectStore('sync_queue', { keyPath: 'id' });
            },
        });
        this.initialized = true;
    }

    private async getDB() {
        if (!this.db) await this.init();
        return this.db!;
    }

    // --- Encryption Helpers ---

    async encryptData(data: any, indexFields: string[] = []): Promise<EncryptedData> {
        // If already encrypted, return as is (checking for _encrypted property)
        if (data._encrypted && data._iv) {
            return data as EncryptedData;
        }

        const { iv, data: encrypted } = await cryptoService.encrypt(data);
        const result: EncryptedData = {
            id: data.id,
            _encrypted: encrypted,
            _iv: iv
        };
        // Copy indexable fields to the root level so IndexedDB indexes work
        indexFields.forEach(field => {
            if (data[field] !== undefined) result[field] = data[field];
        });
        return result;
    }

    private async decryptData(entry: EncryptedData | undefined): Promise<any> {
        if (!entry) return null;
        if (!entry._encrypted) return entry; // Migration: handle unencrypted data if any

        const decrypted = await cryptoService.decrypt(entry._encrypted, entry._iv);
        // Merge back index fields just in case, though decrypted data should be complete
        return { ...decrypted, ...entry };
    }

    // --- Patients (Encrypted) ---

    async getPatient(id: string) {
        const db = await this.getDB();
        const entry = await db.get('patients', id);
        return this.decryptData(entry);
    }

    async getAllPatients() {
        const db = await this.getDB();
        const entries = await db.getAll('patients');
        return Promise.all(entries.map(e => this.decryptData(e)));
    }

    async savePatient(patient: any) {
        const db = await this.getDB();
        // Indexable fields for Patients: lastName
        const entry = await this.encryptData(patient, ['lastName', 'firstName', 'medicalRecordNumber']);
        return db.put('patients', entry);
    }

    // --- Appointments (Unencrypted for now) ---

    async getAppointment(id: string) {
        const db = await this.getDB();
        return db.get('appointments', id);
    }

    async getAllAppointments() {
        const db = await this.getDB();
        return db.getAll('appointments');
    }

    async saveAppointment(apt: any) {
        const db = await this.getDB();
        return db.put('appointments', apt);
    }

    // --- Notes (Encrypted) ---

    async getNote(id: string) {
        const db = await this.getDB();
        const entry = await db.get('notes', id);
        return this.decryptData(entry);
    }

    async getAllNotes() {
        const db = await this.getDB();
        const entries = await db.getAll('notes');
        return Promise.all(entries.map(e => this.decryptData(e)));
    }

    async saveNote(note: any) {
        const db = await this.getDB();
        // Indexable fields for Notes: patientId
        const entry = await this.encryptData(note, ['patientId', 'status', 'createdAt']);
        return db.put('notes', entry);
    }

    // --- Billing (Unencrypted for now) ---

    async getBillingEntries() {
        const db = await this.getDB();
        return db.getAll('billing');
    }

    async saveBillingEntry(entry: any) {
        const db = await this.getDB();
        return db.put('billing', entry);
    }

    // --- Sync Queue Operations ---

    async queueChange(change: any) {
        const db = await this.getDB();
        return db.put('sync_queue', change);
    }

    async getPendingChanges() {
        const db = await this.getDB();
        return db.getAll('sync_queue');
    }

    async removePendingChange(id: string) {
        const db = await this.getDB();
        return db.delete('sync_queue', id);
    }
}

export const db = new Database();
