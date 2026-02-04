<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  
  let appointments: any[] = [];
  let loading = true;
  let currentDate = new Date();
  
  $: formattedDate = currentDate.toLocaleDateString('en-US', { 
    weekday: 'long', 
    year: 'numeric', 
    month: 'long', 
    day: 'numeric' 
  });

  async function loadAppointments() {
    loading = true;
    try {
      // Mock start/end of day query
      const start = new Date(currentDate);
      start.setHours(0,0,0,0);
      const end = new Date(currentDate);
      end.setHours(23,59,59,999);
      
      const res = await api.get(`/appointments?start=${start.toISOString()}&end=${end.toISOString()}`);
      appointments = res.appointments;
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function nextDay() {
    const d = new Date(currentDate);
    d.setDate(d.getDate() + 1);
    currentDate = d;
    loadAppointments();
  }

  function prevDay() {
    const d = new Date(currentDate);
    d.setDate(d.getDate() - 1);
    currentDate = d;
    loadAppointments();
  }

  function today() {
    currentDate = new Date();
    loadAppointments();
  }

  onMount(() => {
    loadAppointments();
  });
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Schedule</h1>
      <p class="text-muted-foreground">Manage appointments and availability.</p>
    </div>
    <div class="flex items-center gap-2">
      <button 
        on:click={today}
        class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2">
        Today
      </button>
       <div class="flex items-center rounded-md border bg-background">
          <button on:click={prevDay} class="p-2 hover:bg-accent hover:text-accent-foreground border-r">
              &lt;
          </button>
          <button on:click={nextDay} class="p-2 hover:bg-accent hover:text-accent-foreground">
              &gt;
          </button>
       </div>
      <button class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2">
        New Appointment
      </button>
    </div>
  </div>

  <div class="rounded-xl border bg-card text-card-foreground shadow">
      <div class="p-6 border-b">
          <h2 class="text-lg font-semibold">{formattedDate}</h2>
      </div>
      <div class="p-0">
          {#if loading}
              <div class="p-8 text-center">Loading schedule...</div>
          {:else if appointments.length === 0}
              <div class="p-12 text-center text-muted-foreground">
                  No appointments scheduled for this day.
              </div>
          {:else}
              <div class="divide-y">
                  {#each appointments as apt}
                      <div class="flex items-center p-4 hover:bg-muted/50 transition-colors">
                          <div class="w-24 text-sm font-medium text-muted-foreground">
                              {new Date(apt.start_time).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit'})}
                          </div>
                          <div class="flex-1">
                              <div class="font-medium">{apt.patient_id} <!-- Should be patient name, need joined data --></div>
                              <div class="text-sm text-muted-foreground">{apt.appointment_type} • {apt.duration} min</div>
                          </div>
                          <div>
                               <span class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground">
                                  {apt.status}
                               </span>
                          </div>
                          <div class="ml-4">
                              <button class="text-sm text-primary hover:underline">View</button>
                          </div>
                      </div>
                  {/each}
              </div>
          {:/if}
      </div>
  </div>
</div>
