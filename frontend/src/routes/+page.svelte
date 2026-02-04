<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  
  let stats = {
    patient_count: 0,
    appointment_count: 0,
    todays_appointments: [] as any[]
  };
  let loading = true;

  onMount(async () => {
    try {
      // In a real app we'd fetch this from a dashboard endpoint
      // Mocking for now as we don't have a dedicated dashboard endpoint
      const patients = await api.get('/patients?limit=5');
      const appointments = await api.get('/appointments');
      
      stats = {
        patient_count: patients.total,
        appointment_count: appointments.appointments.length, // Mock total
        todays_appointments: appointments.appointments.slice(0, 5)
      };
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="space-y-6">
  <div>
    <h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
    <p class="text-muted-foreground">Overview of your practice today.</p>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
    <div class="rounded-xl border bg-card text-card-foreground shadow">
      <div class="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
        <h3 class="tracking-tight text-sm font-medium">Total Patients</h3>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" class="h-4 w-4 text-muted-foreground"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/></svg>
      </div>
      <div class="p-6 pt-0">
        <div class="text-2xl font-bold">{loading ? '-' : stats.patient_count}</div>
        <p class="text-xs text-muted-foreground">+2 from last week</p>
      </div>
    </div>
    <div class="rounded-xl border bg-card text-card-foreground shadow">
      <div class="p-6 flex flex-row items-center justify-between space-y-0 pb-2">
        <h3 class="tracking-tight text-sm font-medium">Appointments</h3>
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" class="h-4 w-4 text-muted-foreground"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/></svg>
      </div>
      <div class="p-6 pt-0">
        <div class="text-2xl font-bold">{loading ? '-' : stats.appointment_count}</div>
        <p class="text-xs text-muted-foreground">For today</p>
      </div>
    </div>
  </div>

  <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
    <div class="col-span-4 rounded-xl border bg-card text-card-foreground shadow">
      <div class="flex flex-col space-y-1.5 p-6">
        <h3 class="font-semibold leading-none tracking-tight">Today's Schedule</h3>
        <p class="text-sm text-muted-foreground">You have {stats.todays_appointments.length} appointments remaining.</p>
      </div>
      <div class="p-6 pt-0">
        <div class="space-y-8">
          {#if loading}
            <div class="text-sm">Loading...</div>
          {:else if stats.todays_appointments.length === 0}
            <div class="text-sm text-muted-foreground">No appointments today.</div>
          {:else}
            {#each stats.todays_appointments as apt}
              <div class="flex items-center">
                <div class="space-y-1">
                  <p class="text-sm font-medium leading-none">{new Date(apt.start_time).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}</p>
                  <p class="text-sm text-muted-foreground">{apt.duration} min</p>
                </div>
                <div class="ml-4 space-y-1">
                  <p class="text-sm font-medium leading-none">{apt.appointment_type}</p>
                  <p class="text-sm text-muted-foreground">{apt.reason_for_visit}</p>
                </div>
                <div class="ml-auto font-medium">
                    <span class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80">
                        {apt.status}
                    </span>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>
