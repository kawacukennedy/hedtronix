<script lang="ts">
    import { onMount } from 'svelte';
    import * as Card from "$lib/components/ui/card";
    import { Users, Calendar, Activity, DollarSign, RefreshCw } from 'lucide-svelte';
    import { Button } from "$lib/components/ui/button";
    import { syncStore } from '$lib/stores/sync';
    import { db } from '$lib/db/indexed-db';
    import { auth } from '$lib/stores/auth';

    let stats = {
        patients: 0,
        appointments: 0,
        notes: 0,
        revenue: 0 // In a real app, this would be calculated from billing entries
    };

    let recentActivity: any[] = [];

    onMount(async () => {
        // Load stats from local DB (Offline First!)
        try {
            const patients = await db.getAllPatients();
            const appointments = await db.getAllAppointments();
            const notes = await db.getAllNotes();
            
            // Calculate today's appointments
            const today = new Date().toISOString().split('T')[0];
            const todaysAppts = appointments.filter((a: any) => a.startTime.startsWith(today));

            stats = {
                patients: patients.length,
                appointments: todaysAppts.length,
                notes: notes.filter((n: any) => n.status === 'DRAFT').length,
                revenue: 0 
            };
            
            // Mock recent activity for now, or derive from sync logs if we tracked them
            recentActivity = [
                { text: 'System Initialized', time: new Date().toLocaleTimeString() }
            ];

        } catch (e) {
            console.error("Failed to load dashboard stats", e);
        }
    });

    function refresh() {
        syncStore.sync();
    }
</script>

<div class="flex flex-col gap-6 p-6">
    <div class="flex items-center justify-between">
        <div>
            <h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
            <p class="text-muted-foreground">Welcome back, {$auth.user?.name || 'Doctor'}</p>
        </div>
        <div class="flex items-center gap-2">
            <span class="text-sm text-muted-foreground">
                Status: <span class={`font-medium ${$syncStore.status === 'ONLINE' ? 'text-green-600' : 'text-amber-600'}`}>{$syncStore.status}</span> 
                {#if $syncStore.lastSync}
                • Last sync: {$syncStore.lastSync.toLocaleTimeString()}
                {/if}
            </span>
            <Button variant="outline" size="icon" on:click={refresh} disabled={$syncStore.status === 'SYNCING'}>
                <RefreshCw class={`h-4 w-4 ${$syncStore.status === 'SYNCING' ? 'animate-spin' : ''}`} />
            </Button>
        </div>
    </div>

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Total Patients</Card.Title>
                <Users class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">{stats.patients}</div>
                <p class="text-xs text-muted-foreground">Registered in local database</p>
            </Card.Content>
        </Card.Root>
        <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Appointments Today</Card.Title>
                <Calendar class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">{stats.appointments}</div>
                <p class="text-xs text-muted-foreground">Scheduled for today</p>
            </Card.Content>
        </Card.Root>
        <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Draft Notes</Card.Title>
                <Activity class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">{stats.notes}</div>
                <p class="text-xs text-muted-foreground">Requires signature</p>
            </Card.Content>
        </Card.Root>
        <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Pending Sync</Card.Title>
                <RefreshCw class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">{$syncStore.pendingCount}</div>
                <p class="text-xs text-muted-foreground">Changes queued offline</p>
            </Card.Content>
        </Card.Root>
    </div>

    <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card.Root class="col-span-4">
            <Card.Header>
                <Card.Title>Recent Activity</Card.Title>
            </Card.Header>
            <Card.Content>
                <div class="space-y-4">
                    {#each recentActivity as activity}
                        <div class="flex items-center">
                            <div class="ml-4 space-y-1">
                                <p class="text-sm font-medium leading-none">{activity.text}</p>
                                <p class="text-sm text-muted-foreground">{activity.time}</p>
                            </div>
                        </div>
                    {/each}
                    {#if recentActivity.length === 0}
                        <p class="text-sm text-muted-foreground">No recent activity.</p>
                    {/if}
                </div>
            </Card.Content>
        </Card.Root>
        <Card.Root class="col-span-3">
             <Card.Header>
                <Card.Title>Quick Actions</Card.Title>
            </Card.Header>
            <Card.Content class="grid gap-2">
                 <Button variant="outline" class="w-full justify-start" href="/patients/new">
                    <Users class="mr-2 h-4 w-4" /> Register Patient
                 </Button>
                 <Button variant="outline" class="w-full justify-start" href="/appointments/new">
                    <Calendar class="mr-2 h-4 w-4" /> Schedule Appt
                 </Button>
                 <Button variant="outline" class="w-full justify-start" href="/notes/new">
                    <Activity class="mr-2 h-4 w-4" /> Write Note
                 </Button>
            </Card.Content>
        </Card.Root>
    </div>
</div>
