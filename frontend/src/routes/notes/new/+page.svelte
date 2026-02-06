<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { syncStore } from '$lib/stores/sync';
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { ArrowLeft, Save } from 'lucide-svelte';
    import { page } from '$app/stores';
    import RichTextEditor from "$lib/components/RichTextEditor.svelte";

    let loading = false;
    let patients: any[] = [];
    
    // Note data
    let note = {
        patientId: $page.url.searchParams.get('patientId') || '',
        noteType: 'PROGRESS_NOTE',
        content: '', // Rich text placeholder
        status: 'DRAFT',
        createdAt: new Date().toISOString()
    };

    onMount(async () => {
        patients = await db.getAllPatients();
    });

    async function handleSave() {
        loading = true;
        try {
            await syncStore.returnChange('ClinicalNote', crypto.randomUUID(), 'CREATE', {
                ...note,
                authorId: $auth.user?.id || 'unknown',
                updatedAt: new Date().toISOString()
            });
            goto('/notes');
        } catch (e) {
            console.error(e);
            alert('Failed to save note');
        } finally {
            loading = false;
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4">
        <Button variant="ghost" size="icon" href="/notes">
            <ArrowLeft class="h-4 w-4" />
        </Button>
        <div>
            <h1 class="text-3xl font-bold tracking-tight">New Clinical Note</h1>
            <p class="text-muted-foreground">Create documentation</p>
        </div>
    </div>

    <Card.Root>
        <Card.Content class="space-y-6 pt-6">
            <div class="grid gap-4 md:grid-cols-2">
                <div class="space-y-2">
                    <Label for="patient">Patient</Label>
                    <select id="patient" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={note.patientId}>
                        <option value="">Select Patient...</option>
                        {#each patients as p}
                            <option value={p.id}>{p.lastName}, {p.firstName} ({p.medicalRecordNumber})</option>
                        {/each}
                    </select>
                </div>
                <div class="space-y-2">
                    <Label for="type">Note Type</Label>
                    <select id="type" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={note.noteType}>
                        <option value="PROGRESS_NOTE">Progress Note</option>
                        <option value="CONSULTATION">Consultation</option>
                        <option value="PROCEDURE_NOTE">Procedure Note</option>
                        <option value="DISCHARGE_SUMMARY">Discharge Summary</option>
                    </select>
                </div>
            </div>

            <div class="space-y-2">
                <Label for="content">Content</Label>
                <div class="min-h-[400px] w-full border rounded-md">
                     <RichTextEditor bind:content={note.content} placeholder="Enter clinical note here..." />
                </div>
                <p class="text-xs text-muted-foreground">Rich text supported.</p>
            </div>

            <div class="flex justify-end gap-4 pt-4">
                <Button variant="outline" href="/notes">Cancel</Button>
                <Button on:click={handleSave} disabled={loading || !note.patientId}>
                    <Save class="mr-2 h-4 w-4" />
                    {#if loading}Saving...{:else}Save Draft{/if}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>
</div>
