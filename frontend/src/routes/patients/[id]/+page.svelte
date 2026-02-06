<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import VitalsEntryForm from "$lib/components/VitalsEntryForm.svelte";
    import VitalsChart from "$lib/components/VitalsChart.svelte";
    import { syncStore } from '$lib/stores/sync';

    import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
    import { User, Activity, FileText, Pill, AlertTriangle } from 'lucide-svelte';
    import Timeline from "$lib/components/Timeline.svelte";

    let patientId = $page.params.id;
    let patient: any = null;
    let loading = true;
    let error: string | null = null;
    let timelineItems: any[] = [];

    onMount(async () => {
        try {
            patient = await db.getPatient(patientId);
            if (!patient) {
                 error = "Patient not found locally. Ensure sync is complete.";
            } else {
                // Load Timeline Data
                const allNotes = await db.getAllNotes();
                const patientNotes = allNotes.filter((n: any) => n.patientId === patientId);

                const allApts = await db.getAllAppointments();
                const patientApts = allApts.filter((a: any) => a.patientId === patientId);

                const notesItems = patientNotes.map((n: any) => ({
                    id: n.id,
                    type: 'NOTE',
                    date: n.createdAt,
                    title: n.noteType.replace('_', ' '),
                    description: 'Clinical documentation created',
                    meta: n
                }));

                const aptItems = patientApts.map((a: any) => ({
                    id: a.id,
                    type: 'APPOINTMENT',
                    date: a.startTime,
                    title: a.appointmentType || 'Appointment',
                    description: `Scheduled with Dr. ${a.providerName || 'Unknown'}`,
                    meta: a
                }));

                const vitalItems = (patient?.vitals || []).map((v: any, i: number) => ({
                    id: `vital-${i}`,
                    type: 'VITAL',
                    date: v.recordedAt,
                    title: 'Vitals Recorded',
                    description: '',
                    meta: v
                }));

                timelineItems = [...notesItems, ...aptItems, ...vitalItems]
                    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());
            }
        } catch (e) {
            error = "Error loading patient";
            console.error(e);
        } finally {
            loading = false;
        }
    });

    async function handleAddVital(event: CustomEvent) {
        const entry = event.detail;
        if (!patient.vitals) patient.vitals = [];
        const newVitals = [...patient.vitals, entry];
        
        // Update patient record
        const updatedPatient = { ...patient, vitals: newVitals, updatedAt: new Date().toISOString() };
        
        try {
            await syncStore.returnChange('Patient', patient.id, 'UPDATE', updatedPatient);
            patient = updatedPatient; // Optimistic update
        } catch (e) {
            console.error(e);
            alert('Failed to save vitals');
        }
    }
</script>

<div class="flex flex-col gap-6 p-6">
    {#if loading}
        <div class="animate-pulse space-y-4">
            <div class="h-12 w-64 bg-gray-200 rounded"></div>
            <div class="h-64 bg-gray-200 rounded"></div>
        </div>
    {:else if error || !patient}
        <div class="p-8 text-center text-red-600 bg-red-50 rounded">
            {error || "Patient not found"}
        </div>
    {:else}
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-3xl font-bold tracking-tight">{patient.firstName} {patient.lastName}</h1>
                <p class="text-muted-foreground">MRN: {patient.medicalRecordNumber} • DOB: {patient.dateOfBirth}</p>
            </div>
            <div class="flex gap-2">
                <Button variant="outline">Edit Profile</Button>
                <Button href={`/appointments/new?patientId=${patient.id}`}>Schedule Appointment</Button>
            </div>
        </div>

        <div class="grid gap-6 md:grid-cols-7">
            <div class="col-span-2 space-y-6">
                <!-- Patient Info Card -->
                <Card.Root>
                    <Card.Header>
                        <Card.Title>Contact Info</Card.Title>
                    </Card.Header>
                    <Card.Content class="space-y-4 text-sm">
                        <div>
                            <span class="font-medium block">Phone</span>
                            {patient.phone || 'N/A'}
                        </div>
                        <div>
                            <span class="font-medium block">Email</span>
                            {patient.email || 'N/A'}
                        </div>
                        <div>
                            <span class="font-medium block">Address</span>
                            {#if patient.address}
                                {patient.address.line1}<br/>
                                {patient.address.city}, {patient.address.state} {patient.address.zip}
                            {:else}
                                N/A
                            {/if}
                        </div>
                    </Card.Content>
                </Card.Root>

                <!-- Alerts Card -->
                <Card.Root>
                    <Card.Header>
                         <Card.Title class="flex items-center gap-2 text-red-600">
                             <AlertTriangle class="h-4 w-4" /> Allergies
                         </Card.Title>
                    </Card.Header>
                    <Card.Content>
                         {#if patient.allergies && patient.allergies.length > 0}
                             <ul class="list-disc pl-4 space-y-1">
                                 {#each patient.allergies as allergy}
                                     <li class="text-red-600 font-medium">{allergy.name || allergy}</li>
                                 {/each}
                             </ul>
                         {:else}
                             <p class="text-sm text-muted-foreground">No known allergies</p>
                         {/if}
                    </Card.Content>
                </Card.Root>
            </div>

            <div class="col-span-5">
                <Tabs defaultValue="history" class="w-full">
                    <TabsList>
                        <TabsTrigger value="history">Medical History</TabsTrigger>
                        <TabsTrigger value="vitals">Vitals</TabsTrigger>
                        <TabsTrigger value="medications">Medications</TabsTrigger>
                        <TabsTrigger value="notes">Clinical Notes</TabsTrigger>
                    </TabsList>
                    
                    <TabsContent value="history" class="mt-4">
                        <!-- ... (existing history content) ... -->
                    </TabsContent>

                    <TabsContent value="vitals" class="mt-4 space-y-6">
                        {#if patient}
                            <Card.Root>
                                <Card.Header>
                                    <Card.Title>Vitals Trends</Card.Title>
                                </Card.Header>
                                <Card.Content>
                                    {#if patient.vitals && patient.vitals.length > 0}
                                        <VitalsChart data={patient.vitals} />
                                    {:else}
                                        <div class="h-[300px] flex items-center justify-center text-muted-foreground border border-dashed rounded-lg">
                                            No vitals recorded
                                        </div>
                                    {/if}
                                </Card.Content>
                            </Card.Root>

                            <VitalsEntryForm on:submit={handleAddVital} />
                            
                            <!-- History Table -->
                            <Card.Root>
                                <Card.Header>
                                    <Card.Title>Vitals History</Card.Title>
                                </Card.Header>
                                <Card.Content>
                                    <div class="border rounded-md">
                                        {#if patient.vitals && patient.vitals.length > 0}
                                        <table class="w-full text-sm">
                                            <thead class="bg-muted">
                                                <tr class="h-10 text-left">
                                                    <th class="pl-4 font-medium">Date/Time</th>
                                                    <th class="font-medium">BP</th>
                                                    <th class="font-medium">HR</th>
                                                    <th class="font-medium">Temp</th>
                                                    <th class="font-medium">SpO2</th>
                                                    <th class="font-medium">Resp</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {#each [...patient.vitals].sort((a,b) => new Date(b.recordedAt).getTime() - new Date(a.recordedAt).getTime()) as v}
                                                <tr class="h-10 border-t hover:bg-muted/50">
                                                    <td class="pl-4">{new Date(v.recordedAt).toLocaleString()}</td>
                                                    <td>{v.systolic}/{v.diastolic}</td>
                                                    <td>{v.heartRate}</td>
                                                    <td>{v.temperature}°F</td>
                                                    <td>{v.spO2}%</td>
                                                    <td>{v.respiratoryRate}</td>
                                                </tr>
                                                {/each}
                                            </tbody>
                                        </table>
                                        {:else}
                                            <div class="p-4 text-center text-muted-foreground">No records</div>
                                        {/if}
                                    </div>
                                </Card.Content>
                            </Card.Root>
                        {/if}
                    </TabsContent>
                    
                    <TabsContent value="medications" class="mt-4">
                        <!-- ... (existing meds content) ... -->
                    </TabsContent>
                    
                     <TabsContent value="notes" class="mt-4">
                        <div class="flex justify-end mb-4">
                            <Button size="sm" href={`/notes/new?patientId=${patient.id}`}>
                                <FileText class="mr-2 h-4 w-4" /> New Note
                            </Button>
                        </div>
                         <!-- Notes List would go here, fetching from db.getNotesByPatient(id) -->
                         <p class="text-sm text-muted-foreground">Notes loading...</p>
                    </TabsContent>
                </Tabs>
            </div>
        </div>
    {/if}
</div>
