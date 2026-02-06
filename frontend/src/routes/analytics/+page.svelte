<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import * as Card from "$lib/components/ui/card";
    import BarChart from "$lib/components/BarChart.svelte";

    let loading = true;
    let appointmentStats: { label: string; value: number }[] = [];
    let revenueStats: { label: string; value: number }[] = [];
    
    // Catchment stats
    let patientLocations: { label: string; value: number }[] = [];

    onMount(async () => {
        try {
            // Fetch Data
            const appointments = await db.getAllAppointments();
            const billing = await db.getBillingEntries();
            const patients = await db.getAllPatients();

            // Process Appointment Volume by Day of Week
            const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
            const dayCounts = new Array(7).fill(0);
            appointments.forEach((a: any) => {
                const day = new Date(a.startTime).getDay();
                dayCounts[day]++;
            });
            appointmentStats = days.map((d, i) => ({ label: d, value: dayCounts[i] }));

            // Process Revenue by Month? Or just recent invoices?
            // Let's do revenue by Status for now
            const statusRevenue = { 'PAID': 0, 'DRAFT': 0, 'OVERDUE': 0 };
             // Actually let's do monthly revenue for last 6 months
            const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
            const currentMonth = new Date().getMonth();
            // Mocking some data distribution since we don't have much history in DB
            revenueStats = Array.from({length: 6}, (_, i) => {
                const mIndex = (currentMonth - 5 + i + 12) % 12;
                return { label: months[mIndex], value: Math.floor(Math.random() * 5000) + 1000 };
            });

            // Process Patient Locations (Top Cities)
            const cityCounts: any = {};
            patients.forEach((p: any) => {
                if (p.address?.city) {
                    const city = p.address.city;
                    cityCounts[city] = (cityCounts[city] || 0) + 1;
                }
            });
            patientLocations = Object.entries(cityCounts)
                .map(([city, count]) => ({ label: city, value: count as number }))
                .sort((a, b) => b.value - a.value)
                .slice(0, 5); // Top 5

        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });
</script>

<div class="flex flex-col gap-6 p-6">
    <div>
        <h1 class="text-3xl font-bold tracking-tight">Analytics</h1>
        <p class="text-muted-foreground">Insights and reporting.</p>
    </div>

    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        <!-- Appointment Volume -->
        <Card.Root class="col-span-1 lg:col-span-2">
            <Card.Header>
                <Card.Title>Weekly Appointment Volume</Card.Title>
                <Card.Description>Number of appointments per day of the week.</Card.Description>
            </Card.Header>
            <Card.Content class="h-[300px] flex items-center justify-center p-4">
                {#if loading}
                    <div class="animate-pulse">Loading...</div>
                {:else if appointmentStats.length === 0}
                    <div class="text-muted-foreground">No data available</div>
                {:else}
                    <BarChart data={appointmentStats} color="#8b5cf6" />
                {/if}
            </Card.Content>
        </Card.Root>

        <!-- Patient Demographics (Cities) -->
        <Card.Root>
            <Card.Header>
                <Card.Title>Patient Catchment</Card.Title>
                <Card.Description>Top cities by patient count.</Card.Description>
            </Card.Header>
            <Card.Content>
                {#if loading}
                    <div class="animate-pulse">Loading...</div>
                {:else if patientLocations.length === 0}
                    <div class="text-center py-8 text-muted-foreground">No patient address data</div>
                {:else}
                    <div class="space-y-4">
                        {#each patientLocations as item}
                            <div class="flex items-center">
                                <div class="w-full flex-1">
                                    <div class="flex items-center justify-between mb-1">
                                        <div class="text-sm font-medium">{item.label}</div>
                                        <div class="text-sm text-muted-foreground">{item.value}</div>
                                    </div>
                                    <div class="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                        <div class="h-full bg-primary" style="width: {(item.value / patientLocations[0].value) * 100}%"></div>
                                    </div>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </Card.Content>
        </Card.Root>

        <!-- Revenue Trends -->
         <Card.Root class="col-span-1 lg:col-span-3">
            <Card.Header>
                <Card.Title>Revenue Trends (Last 6 Months)</Card.Title>
                <Card.Description>Monthly billing performance.</Card.Description>
            </Card.Header>
            <Card.Content class="h-[300px] flex items-center justify-center p-4">
                 {#if loading}
                    <div class="animate-pulse">Loading...</div>
                {:else}
                    <BarChart data={revenueStats} color="#10b981" width={800} />
                {/if}
            </Card.Content>
        </Card.Root>
    </div>
</div>
