<script lang="ts">
  import { onMount } from 'svelte';
  import { db } from '$lib/db/indexed-db';
    import { Button } from "$lib/components/ui/button";
    import Calendar from "$lib/components/Calendar.svelte";
    import { goto } from '$app/navigation';

    let events: any[] = [];
    let loading = true;
    let currentDate = new Date();
    let view: 'day' | 'week' | 'month' = 'day';

    async function loadAppointments() {
        loading = true;
        try {
            const allApts = await db.getAllAppointments();
            const allPatients = await db.getAllPatients();
            const patientMap = new Map(allPatients.map(p => [p.id, `${p.firstName} ${p.lastName}`]));

            events = allApts.map((a: any) => ({
                ...a,
                patientName: patientMap.get(a.patientId) || 'Unknown Patient'
            }));

        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        loadAppointments();
    });

    function handleDateChange(e: CustomEvent) {
        currentDate = e.detail;
    }

    function handleViewChange(e: CustomEvent) {
        view = e.detail;
    }
    
    function handleEventClick(e: CustomEvent) {
        goto(`/appointments/${e.detail.id}`);
    }

    function handleSlotClick(e: CustomEvent) {
        const { date, hour } = e.detail;
        // In real app, open modal with pre-filled time
        const d = new Date(date);
        d.setHours(hour, 0, 0, 0);
        goto(`/appointments/new?startTime=${d.toISOString()}`);
    }

    function handleDateClick(e: CustomEvent) {
        currentDate = e.detail;
        view = 'day'; // Drill down
    }

</script>

<div class="h-[calc(100vh-6rem)] flex flex-col p-6 space-y-4">
    <div class="flex items-center justify-between shrink-0">
        <div>
            <h1 class="text-3xl font-bold tracking-tight">Schedule</h1>
            <p class="text-muted-foreground">Manage clinical appointments.</p>
        </div>
        <Button href="/appointments/new">New Appointment</Button>
    </div>

    <div class="flex-1 min-h-0 bg-card rounded-lg border shadow-sm">
        {#if loading}
             <div class="flex h-full items-center justify-center">Loading...</div>
        {:else}
            <Calendar 
                {events} 
                date={currentDate} 
                {view}
                on:change={handleDateChange}
                on:viewChange={handleViewChange}
                on:eventClick={handleEventClick}
                on:slotClick={handleSlotClick}
                on:dateClick={handleDateClick}
            />
        {/if}
    </div>
</div>
