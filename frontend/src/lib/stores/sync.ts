
import { writable, get } from 'svelte/store';
import { db } from '../db/indexed-db';
import { api } from '../api';
import { auth } from './auth';

interface SyncState {
    status: 'ONLINE' | 'OFFLINE' | 'SYNCING' | 'ERROR';
    lastSync: Date | null;
    pendingCount: number;
}

function createSyncStore() {
    const { subscribe, set, update } = writable<SyncState>({
        status: 'ONLINE', // Assume online initially
        lastSync: null,
        pendingCount: 0
    });

    return {
        subscribe,

        async init() {
            if (typeof window === 'undefined') return;

            await db.init();

            // Initial load of pending count
            const pending = await db.getPendingChanges();
            update(s => ({ ...s, pendingCount: pending.length }));

            // Network listeners
            window.addEventListener('online', () => {
                this.setStatus('ONLINE');
                this.sync();
            });

            window.addEventListener('offline', () => {
                this.setStatus('OFFLINE');
            });

            // Initial sync
            if (navigator.onLine) {
                this.sync();
            } else {
                this.setStatus('OFFLINE');
            }

            // Periodic sync (every minute)
            setInterval(() => {
                if (navigator.onLine) this.sync();
            }, 60000);
        },

        setStatus(status: SyncState['status']) {
            update(s => ({ ...s, status }));
        },

        async sync() {
            const state = get(this);
            if (state.status === 'OFFLINE' || state.status === 'SYNCING') return;

            this.setStatus('SYNCING');

            try {
                // 1. Push pending changes
                const pending = await db.getPendingChanges();
                if (pending.length > 0) {
                    /*
                    // BYPASS: Backend unavailable
                    await api.post('/sync/push', { ... });
                    */
                    console.log('[Mock Sync] Pushing changes:', pending);

                    // Clear queue on success (Mock success)
                    for (const item of pending) {
                        await db.removePendingChange(item.id);
                    }
                }

                // 2. Pull remote changes
                const lastSync = state.lastSync?.toISOString();

                /*
                // BYPASS: Backend unavailable
                const response = await api.get(`/sync/pull?since=${lastSync || ''}`);
                if (response.changes && response.changes.length > 0) {
                    await this.applyRemoteChanges(response.changes);
                }
                */
                console.log('[Mock Sync] Pulling changes... (No backend)');

                update(s => ({
                    ...s,
                    status: 'ONLINE', // Mock Online
                    lastSync: new Date(),
                    pendingCount: 0
                }));

            } catch (error) {
                console.error('Sync failed:', error);
                this.setStatus('ERROR');
            }
        },

        async applyRemoteChanges(changes: any[]) {
            for (const change of changes) {
                switch (change.entity_type) {
                    case 'Patient':
                        await db.savePatient(change.data);
                        break;
                    case 'Appointment':
                        await db.saveAppointment(change.data);
                        break;
                    case 'ClinicalNote':
                        await db.saveNote(change.data);
                        break;
                    case 'BillingEntry':
                        await db.saveBillingEntry(change.data);
                        break;
                }
            }
        },

        // Helper to queue a change locally
        async returnChange(entityType: string, entityId: string, operation: 'CREATE' | 'UPDATE' | 'DELETE', data: any) {

            // 1. Prepare payload for sync queue (Encrypted if sensitive)
            let queuePayload = data;

            if (operation !== 'DELETE') {
                if (entityType === 'Patient') {
                    queuePayload = await db.encryptData(data, ['lastName', 'firstName', 'medicalRecordNumber']);
                } else if (entityType === 'ClinicalNote') {
                    queuePayload = await db.encryptData(data, ['patientId', 'status', 'createdAt']);
                }
                // Billing and Appointments are currently unencrypted per db.ts logic
            }

            const change = {
                id: crypto.randomUUID(),
                entityType,
                entityId,
                operation,
                data: queuePayload,
                timestamp: Date.now()
            };

            await db.queueChange(change);

            // 2. Optimistic update locally (Plaintext - db.save* will handle encryption for storage)
            if (operation === 'DELETE') {
                // TODO: Implement delete methods in db
            } else {
                switch (entityType) {
                    case 'Patient': await db.savePatient(data); break;
                    case 'Appointment': await db.saveAppointment(data); break;
                    case 'ClinicalNote': await db.saveNote(data); break;
                    case 'BillingEntry': await db.saveBillingEntry(data); break;
                }
            }


            const pending = await db.getPendingChanges();
            update(s => ({ ...s, pendingCount: pending.length }));

            // Try to sync immediately if online
            if (navigator.onLine) {
                this.sync();
            }
        }
    };
}

export const syncStore = createSyncStore();
