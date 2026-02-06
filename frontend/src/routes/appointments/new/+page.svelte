<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { syncStore } from '$lib/stores/sync';
    import { goto } from '$app/navigation';
    import { page } from '$app/stores';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { ArrowLeft } from 'lucide-svelte';

    let loading = false;
    let patients: any[] = [];
    
    // Form data
    let appointment = {
        patientId: '',
        providerId: '',
        startTime: '', // Date + Time
        duration: 30,
        appointmentType: 'FOLLOW_UP',
        note: ''
    };

    onMount(async () => {
        patients = await db.getAllPatients();
        
        const startTimeParam = $page.url.searchParams.get('startTime');
        if (startTimeParam) {
            // format for datetime-local input: YYYY-MM-DDTHH:mm
            const d = new Date(startTimeParam);
            const tzOffset = d.getTimezoneOffset() * 60000; // offset in milliseconds
            const localISOTime = (new Date(d.getTime() - tzOffset)).toISOString().slice(0, 16);
            appointment.startTime = localISOTime;
        }
    });

    async function handleSubmit() {
        loading = true;
        try {
            await syncStore.returnChange('Appointment', crypto.randomUUID(), 'CREATE', {
                ...appointment,
                status: 'SCHEDULED',
                // Convert type="" to proper enum case if needed, or stick to consistency
                startTime: new Date(appointment.startTime).toISOString(),
                // Calculate end time or just store duration? Specs say end_time is required in backend model
                endTime: new Date(new Date(appointment.startTime).getTime() + appointment.duration * 60000).toISOString()
            });
            goto('/appointments');
        } catch (e) {
            console.error(e);
            alert('Failed to schedule appointment');
        } finally {
            loading = false;
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-2xl mx-auto">
    <div class="flex items-center gap-4">
        <Button variant="ghost" size="icon" href="/appointments">
            <ArrowLeft class="h-4 w-4" />
        </Button>
        <div>
            <h1 class="text-3xl font-bold tracking-tight">New Appointment</h1>
            <p class="text-muted-foreground">Schedule a visit</p>
        </div>
    </div>

    <Card.Root>
        <Card.Content class="space-y-6 pt-6">
            <div class="space-y-2">
                <Label for="patient">Patient</Label>
                <select id="patient" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={appointment.patientId} required>
                    <option value="">Select Patient...</option>
                    {#each patients as p}
                        <option value={p.id}>{p.lastName}, {p.firstName} ({p.medicalRecordNumber})</option>
                    {/each}
                </select>
            </div>

            <div class="space-y-2">
                <Label for="provider">Provider</Label>
                <select id="provider" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={appointment.providerId}>
                    <option value="">Select Provider...</option>
                    <option value="prov-1">Dr. Smith (General)</option>
                    <option value="prov-2">Dr. Jones (Cardiology)</option>
                    <option value="prov-3">Nurse Ratched</option>
                </select>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
                <div class="space-y-2">
                    <Label for="time">Start Time</Label>
                    <Input id="time" type="datetime-local" bind:value={appointment.startTime} required />
                </div>
                <div class="space-y-2">
                    <Label for="duration">Duration (mins)</Label>
                    <select id="duration" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={appointment.duration}>
                        <option value={15}>15 Minutes</option>
                        <option value={30}>30 Minutes</option>
                        <option value={45}>45 Minutes</option>
                        <option value={60}>1 Hour</option>
                    </select>
                </div>
            </div>

            <div class="space-y-2">
                <Label for="type">Type</Label>
                <select id="type" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={appointment.appointmentType}>
                    <option value="FOLLOW_UP">Follow-up</option>
                    <option value="NEW_PATIENT">New Patient</option>
                    <option value="CONSULTATION">Consultation</option>
                    <option value="PROCEDURE">Procedure</option>
                </select>
            </div>

            <div class="space-y-2">
                <Label for="note">Reason / Notes</Label>
                <textarea 
                    id="note" 
                    class="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    bind:value={appointment.note}
                ></textarea>
            </div>

            <div class="flex justify-end gap-4 pt-4">
                <Button variant="outline" href="/appointments">Cancel</Button>
                <Button on:click={handleSubmit} disabled={loading || !appointment.patientId || !appointment.startTime}>
                    {#if loading}Scheduling...{:else}Schedule Appointment{/if}
                </Button>
            </div>
        </Card.Content>
    </Card.Root>
</div>
