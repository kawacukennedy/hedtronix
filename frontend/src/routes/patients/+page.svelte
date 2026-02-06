<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { type User } from '$lib/stores/auth'; // Reusing type for clean code, though Patient is different
    import { db } from '$lib/db/indexed-db';
    import { syncStore } from '$lib/stores/sync'; // Trigger sync on load?
    
    // UI Components
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Table from "$lib/components/ui/table";
    import * as Card from "$lib/components/ui/card";
    import { Search, UserPlus, Filter } from 'lucide-svelte';

    let patients: any[] = [];
    let loading = true;
    let searchQuery = '';

    onMount(async () => {
        await loadPatients();
        // Listen for sync changes? Ideally syncStore would emit an event or we poll
        // For now, reload on mount is good enough for MVP
    });

    async function loadPatients() {
        loading = true;
        try {
            patients = await db.getAllPatients();
        } catch (e) {
            console.error("Failed to load patients", e);
        } finally {
            loading = false;
        }
    }

    $: filteredPatients = patients.filter(p => {
        const query = searchQuery.toLowerCase();
        return (
            p.firstName.toLowerCase().includes(query) ||
            p.lastName.toLowerCase().includes(query) ||
            p.medicalRecordNumber.toLowerCase().includes(query)
        );
    });
</script>

<div class="flex flex-col gap-6 p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight">Patients</h1>
            <p class="text-muted-foreground">Manage patient records</p>
        </div>
        <Button href="/patients/new">
            <UserPlus class="mr-2 h-4 w-4" />
            Add Patient
        </Button>
    </div>

    <Card.Root>
        <Card.Header>
            <div class="flex items-center gap-4">
                <div class="relative flex-1 max-w-sm">
                    <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input 
                        type="search" 
                        placeholder="Search by name or MRN..." 
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
                <div class="text-center py-4">Loading patients...</div>
            {:else if filteredPatients.length === 0}
                 <div class="text-center py-8 text-muted-foreground">
                    No patients found.
                </div>
            {:else}
            <div class="overflow-x-auto">
                <Table.Root>
                    <Table.Header>
                        <Table.Row>
                            <Table.Head>Name</Table.Head>
                            <Table.Head>MRN</Table.Head>
                            <Table.Head>DOB</Table.Head>
                            <Table.Head>Status</Table.Head>
                            <Table.Head class="text-right">Actions</Table.Head>
                        </Table.Row>
                    </Table.Header>
                    <Table.Body>
                        {#each filteredPatients as patient}
                            <Table.Row>
                                <Table.Cell class="font-medium">
                                    <a href={`/patients/${patient.id}`} class="hover:underline">
                                        {patient.lastName}, {patient.firstName}
                                    </a>
                                </Table.Cell>
                                <Table.Cell>{patient.medicalRecordNumber}</Table.Cell>
                                <Table.Cell>{patient.dateOfBirth}</Table.Cell>
                                <Table.Cell>
                                    <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-green-100 text-green-800">
                                        Active
                                    </span>
                                </Table.Cell>
                                <Table.Cell class="text-right">
                                    <Button variant="ghost" size="sm" href={`/patients/${patient.id}`}>View</Button>
                                </Table.Cell>
                            </Table.Row>
                        {/each}
                    </Table.Body>
                </Table.Root>
            </div>
            {/if}
        </Card.Content>
    </Card.Root>
</div>
