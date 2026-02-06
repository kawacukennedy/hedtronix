<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { ArrowLeft } from 'lucide-svelte';
    import { syncStore } from '$lib/stores/sync';
    import { goto } from '$app/navigation';

    let loading = false;
    // Patient form data
    let patient = {
        firstName: '',
        lastName: '',
        dateOfBirth: '',
        medicalRecordNumber: `MRN-${Math.floor(Math.random() * 100000)}`, // Auto-generate for now
        phone: '',
        email: '',
        address: {
            line1: '',
            city: '',
            state: '',
            zip: ''
        }
    };

    async function handleSubmit() {
        loading = true;
        try {
            await syncStore.returnChange('Patient', crypto.randomUUID(), 'CREATE', patient);
            goto('/patients');
        } catch (e) {
            console.error(e);
            alert('Failed to save patient');
        } finally {
            loading = false;
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-2xl mx-auto">
    <div class="flex items-center gap-4">
        <Button variant="ghost" size="icon" href="/patients">
            <ArrowLeft class="h-4 w-4" />
        </Button>
        <div>
            <h1 class="text-3xl font-bold tracking-tight">New Patient</h1>
            <p class="text-muted-foreground">Register a new patient record</p>
        </div>
    </div>

    <Card.Root>
        <Card.Content class="space-y-6 pt-6">
            <div class="grid gap-4 md:grid-cols-2">
                <div class="space-y-2">
                    <Label for="firstName">First Name</Label>
                    <Input id="firstName" bind:value={patient.firstName} required />
                </div>
                <div class="space-y-2">
                    <Label for="lastName">Last Name</Label>
                    <Input id="lastName" bind:value={patient.lastName} required />
                </div>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
                <div class="space-y-2">
                    <Label for="dob">Date of Birth</Label>
                    <Input id="dob" type="date" bind:value={patient.dateOfBirth} required />
                </div>
                <div class="space-y-2">
                    <Label for="mrn">MRN</Label>
                    <Input id="mrn" bind:value={patient.medicalRecordNumber} />
                </div>
            </div>

            <div class="space-y-2">
                <Label for="phone">Phone Number</Label>
                <Input id="phone" type="tel" bind:value={patient.phone} />
            </div>
            
            <div class="space-y-2">
                <Label for="email">Email</Label>
                <Input id="email" type="email" bind:value={patient.email} />
            </div>

            <div class="space-y-2">
                <h3 class="font-medium">Address</h3>
                <Input placeholder="Address Line 1" bind:value={patient.address.line1} class="mb-2" />
                <div class="grid grid-cols-3 gap-2">
                    <Input placeholder="City" bind:value={patient.address.city} />
                    <Input placeholder="State" bind:value={patient.address.state} />
                    <Input placeholder="ZIP" bind:value={patient.address.zip} />
                </div>
            </div>

            <div class="flex justify-end gap-4 pt-4">
                <Button variant="outline" href="/patients">Cancel</Button>
                <Button on:click={handleSubmit} disabled={loading}>
                    {#if loading}Saving...{:else}Create Patient{/if}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>
</div>
