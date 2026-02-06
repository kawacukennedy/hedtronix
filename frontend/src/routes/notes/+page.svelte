<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Card from "$lib/components/ui/card";
    import { Search, Plus, Filter } from 'lucide-svelte';

    let notes: any[] = [];
    let loading = true;
    let searchQuery = '';
    let patientsMap = new Map();

    onMount(async () => {
        try {
            notes = await db.getAllNotes();
            const patients = await db.getAllPatients();
            patientsMap = new Map(patients.map(p => [p.id, `${p.lastName}, ${p.firstName}`]));
            
            // Map patient names
            notes = notes.map(n => ({
                ...n,
                patientName: patientsMap.get(n.patientId) || 'Unknown Patient'
            })).sort((a,b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());

        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    $: filteredNotes = notes.filter(n => {
        const query = searchQuery.toLowerCase();
        return (
            n.patientName.toLowerCase().includes(query) ||
            n.noteType?.toLowerCase().includes(query) ||
            n.status?.toLowerCase().includes(query)
        );
    });
</script>

<div class="flex flex-col gap-6 p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight">Clinical Notes</h1>
            <p class="text-muted-foreground">Documentation and records</p>
        </div>
        <Button href="/notes/new">
            <Plus class="mr-2 h-4 w-4" />
            New Note
        </Button>
    </div>

    <Card.Root>
        <Card.Header>
            <div class="flex items-center gap-4">
                <div class="relative flex-1 max-w-sm">
                    <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input 
                        type="search" 
                        placeholder="Search notes..." 
                        class="pl-8"
                        bind:value={searchQuery}
                    />
                </div>
                <Button variant="outline" size="icon">
                    <Filter class="h-4 w-4" />
                </Button>
            </div>
        </Card.Header>
        <Card.Content>
            {#if loading}
                 <div class="p-8 text-center animate-pulse">Loading notes...</div>
            {:else if filteredNotes.length === 0}
                 <div class="p-12 text-center text-muted-foreground">
                    No clinical notes found.
                </div>
            {:else}
                <div class="divide-y border rounded-md">
                    {#each filteredNotes as note}
                        <div class="flex items-center p-4 hover:bg-muted/50 transition-colors">
                            <div class="flex-1 space-y-1">
                                <div class="flex items-center gap-2">
                                     <a href={`/notes/${note.id}`} class="font-semibold hover:underline">{note.patientName}</a>
                                     <span class="text-xs text-muted-foreground px-2 py-0.5 rounded-full bg-slate-100 border">
                                         {note.noteType || 'General'}
                                     </span>
                                </div>
                                <div class="text-sm text-muted-foreground">
                                    Created {new Date(note.createdAt).toLocaleDateString()}
                                </div>
                            </div>
                            <div class="flex items-center gap-4">
                                <span class={`text-xs px-2 py-1 rounded-full font-medium
                                    ${note.status === 'SIGNED' ? 'bg-green-100 text-green-800' : 'bg-yellow-100 text-yellow-800'}`}>
                                    {note.status}
                                </span>
                                <Button variant="ghost" size="sm" href={`/notes/${note.id}`}>View</Button>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </Card.Content>
    </Card.Root>
</div>
