<script lang="ts">
    import { page } from '$app/stores';
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { Button } from "$lib/components/ui/button";
    import { Calendar, User, Clock, FileText, CheckCircle, XCircle } from 'lucide-svelte';
    import * as Card from "$lib/components/ui/card";

    let aptId = $page.params.id;
    let appointment: any = null;
    let patient: any = null;
    let loading = true;

    onMount(async () => {
        try {
            appointment = await db.getAppointment(aptId);
            if (appointment && appointment.patientId) {
                patient = await db.getPatient(appointment.patientId);
            }
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    const formatTime = (iso: string) => {
        if (!iso) return 'N/A';
        return new Date(iso).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'});
    };
    
    const formatDate = (iso: string) => {
         if (!iso) return 'N/A';
        return new Date(iso).toLocaleDateString([], {weekday: 'long', month: 'long', day: 'numeric'});
    };
</script>

<div class="flex flex-col gap-6 p-6 max-w-3xl mx-auto">
    {#if loading}
        <div class="h-64 bg-gray-100 rounded animate-pulse"></div>
    {:else if !appointment}
        <div class="p-8 text-center text-red-600 bg-red-50 rounded">Appointment not found</div>
    {:else}
        <div class="flex items-center justify-between border-b pb-4">
            <div>
                 <h1 class="text-2xl font-bold tracking-tight">Appointment Details</h1>
                 <p class="text-muted-foreground">ID: {appointment.id}</p>
            </div>
            <div class="flex items-center gap-2">
                 <span class="px-3 py-1 bg-blue-100 text-blue-800 rounded-full font-medium text-sm">
                     {appointment.status || 'SCHEDULED'}
                 </span>
            </div>
        </div>

        <div class="grid gap-6">
            <Card.Root>
                <Card.Content class="pt-6 grid gap-6 md:grid-cols-2">
                    <div class="space-y-4">
                        <div class="flex items-start gap-3">
                            <Calendar class="h-5 w-5 text-muted-foreground mt-0.5" />
                            <div>
                                <h3 class="font-medium text-sm text-muted-foreground">Date & Time</h3>
                                <p class="text-lg font-semibold">{formatDate(appointment.startTime)}</p>
                                <p>{formatTime(appointment.startTime)} - {formatTime(appointment.endTime)}</p>
                            </div>
                        </div>
                         <div class="flex items-start gap-3">
                            <User class="h-5 w-5 text-muted-foreground mt-0.5" />
                            <div>
                                <h3 class="font-medium text-sm text-muted-foreground">Patient</h3>
                                <p class="font-semibold text-lg">{patient ? `${patient.lastName}, ${patient.firstName}` : 'Unknown'}</p>
                            </div>
                        </div>
                    </div>
                    
                    <div class="space-y-4">
                         <div class="flex items-start gap-3">
                            <FileText class="h-5 w-5 text-muted-foreground mt-0.5" />
                            <div>
                                <h3 class="font-medium text-sm text-muted-foreground">Reason/Notes</h3>
                                <p>{appointment.note || appointment.reason || 'None'}</p>
                            </div>
                        </div>
                    </div>
                </Card.Content>
                <Card.Footer class="border-t bg-muted/50 p-4 flex justify-end gap-3">
                    <Button variant="outline" class="text-red-600 hover:text-red-700">
                        <XCircle class="mr-2 h-4 w-4" /> Cancel
                    </Button>
                    <Button variant="secondary">
                        Reschedule
                    </Button>
                    <Button>
                        <CheckCircle class="mr-2 h-4 w-4" /> Check In
                    </Button>
                </Card.Footer>
            </Card.Root>
            
            <Card.Root>
                 <Card.Header>
                     <Card.Title class="text-base">Quick Actions</Card.Title>
                 </Card.Header>
                 <Card.Content class="flex gap-3">
                     <Button size="sm" variant="outline" href={`/notes/new?patientId=${appointment.patientId}`}>
                         Create Clinical Note
                     </Button>
                     <Button size="sm" variant="outline" href={`/billing/new?patientId=${appointment.patientId}&date=${appointment.startTime?.split('T')[0]}`}>
                         Add Charge
                     </Button>
                 </Card.Content>
            </Card.Root>
        </div>
    {/if}
</div>
