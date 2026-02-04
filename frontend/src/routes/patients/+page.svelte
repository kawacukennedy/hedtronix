<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  
  let patients: any[] = [];
  let loading = true;
  let query = '';

  async function loadPatients() {
    loading = true;
    try {
      const endpoint = query 
        ? `/patients/search` // Post requires body, let's just stick to list for now or enable search properly
        : `/patients`;
      
      let res;
      if (query) {
         res = await api.post('/patients/search', { query, active_only: true });
      } else {
         res = await api.get('/patients');
      }
      patients = res.patients;
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function handleSearch() {
    loadPatients();
  }

  onMount(() => {
    loadPatients();
  });
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Patients</h1>
      <p class="text-muted-foreground">Manage patient records.</p>
    </div>
    <button class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2">
      Add Patient
    </button>
  </div>

  <div class="flex items-center space-x-2">
    <input 
      class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 max-w-sm" 
      placeholder="Search patients..." 
      bind:value={query}
      on:keydown={(e) => e.key === 'Enter' && handleSearch()}
    />
    <button 
        on:click={handleSearch}
        class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2">
      Search
    </button>
  </div>

  <div class="rounded-md border">
    <div class="relative w-full overflow-auto">
      <table class="w-full caption-bottom text-sm">
        <thead class="[&_tr]:border-b">
          <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">MRN</th>
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Name</th>
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">DOB</th>
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Gender</th>
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Phone</th>
            <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Actions</th>
          </tr>
        </thead>
        <tbody class="[&_tr:last-child]:border-0">
          {#if loading}
             <tr><td colspan="6" class="p-4 text-center">Loading...</td></tr>
          {:else if patients.length === 0}
             <tr><td colspan="6" class="p-4 text-center text-muted-foreground">No patients found.</td></tr>
          {:else}
            {#each patients as patient}
              <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                <td class="p-4 align-middle font-medium">{patient.medical_record_number}</td>
                <td class="p-4 align-middle">{patient.last_name}, {patient.first_name}</td>
                <td class="p-4 align-middle">{patient.date_of_birth}</td>
                <td class="p-4 align-middle">{patient.gender}</td>
                <td class="p-4 align-middle">{patient.phone}</td>
                <td class="p-4 align-middle">
                  <a href={`/patients/${patient.id}`} class="text-primary hover:underline">View</a>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>
