<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { syncStore } from '$lib/stores/sync';
    import { Button } from "$lib/components/ui/button";
    import { User, Calendar, FileText, Lock, PenTool } from 'lucide-svelte';
    import * as Card from "$lib/components/ui/card";

    let noteId = $page.params.id;
    let note: any = null;
    let patient: any = null;
    let loading = true;
    let signing = false;

    onMount(async () => {
        try {
            note = await db.getNote(noteId);
            if (note && note.patientId) {
                patient = await db.getPatient(note.patientId);
            }
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    async function signNote() {
        if (!confirm('Are you sure you want to sign this note? It will become immutable.')) return;
        
        signing = true;
        try {
            // Update local note
            const updated = {
                ...note,
                status: 'SIGNED',
                signedAt: new Date().toISOString()
            };
            
            await syncStore.returnChange('ClinicalNote', noteId, 'UPDATE', updated);
            note = updated; // Optimistic update
        } catch (e) {
            alert('Failed to sign note');
        } finally {
            signing = false;
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-4xl mx-auto">
    {#if loading}
        <div class="h-64 bg-gray-100 rounded animate-pulse"></div>
    {:else if !note}
        <div class="p-8 text-center text-red-600 bg-red-50 rounded">Note not found</div>
    {:else}
        <div class="flex items-center justify-between border-b pb-4">
            <div>
                 <h1 class="text-2xl font-bold tracking-tight">Clinical Note</h1>
                 <p class="text-muted-foreground">{note.noteType} • {new Date(note.createdAt).toLocaleDateString()}</p>
            </div>
            <div class="flex items-center gap-2">
                 {#if note.status === 'SIGNED'}
                    <span class="flex items-center gap-1 px-3 py-1 bg-green-100 text-green-800 rounded-full font-medium text-sm">
                        <Lock class="h-3 w-3" /> Signed
                    </span>
                 {:else}
                    <span class="px-3 py-1 bg-yellow-100 text-yellow-800 rounded-full font-medium text-sm">Draft</span>
                 {/if}
            </div>
        </div>

        <div class="grid gap-6 md:grid-cols-4">
            <div class="md:col-span-3 space-y-6">
                <!-- Note Content -->
                <Card.Root>
                    <Card.Content class="pt-6 min-h-[500px]">
                        <div class="prose max-w-none">
                            {@html note.content}
                        </div>
                    </Card.Content>
                </Card.Root>
            </div>

            <div class="md:col-span-1 space-y-4">
                <!-- Metadata Side Panel -->
                <Card.Root>
                    <Card.Header>
                        <Card.Title class="text-sm">Patient</Card.Title>
                    </Card.Header>
                    <Card.Content>
                        {#if patient}
                            <div class="font-medium">{patient.lastName}, {patient.firstName}</div>
                            <div class="text-sm text-muted-foreground">{patient.medicalRecordNumber}</div>
                        {:else}
                            <div class="text-muted-foreground">Unknown</div>
                        {/if}
                    </Card.Content>
                </Card.Root>
                
                <Card.Root>
                    <Card.Header>
                        <Card.Title class="text-sm">Actions</Card.Title>
                    </Card.Header>
                    <Card.Content class="space-y-2">
                        {#if note.status !== 'SIGNED'}
                            <Button class="w-full" on:click={signNote} disabled={signing}>
                                <PenTool class="mr-2 h-4 w-4" />
                                {signing ? 'Signing...' : 'Sign Note'}
                            </Button>
                            <Button variant="outline" class="w-full">Edit</Button>
                        {:else}
                            <Button variant="outline" class="w-full" disabled>
                                <Lock class="mr-2 h-4 w-4" /> Signed
                            </Button>
                            <Button variant="outline" class="w-full">Add Addendum</Button>
                        {/if}
                    </Card.Content>
                </Card.Root>
            </div>
        </div>
    {/if}
</div>
