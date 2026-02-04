<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { api } from '$lib/api';
  
  let patient: any = null;
  let loading = true;
  let activeTab = 'overview';

  const patientId = $page.params.id;

  onMount(async () => {
    try {
      patient = await api.get(`/patients/${patientId}`);
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  });

  function getAge(dob: string) {
    const today = new Date();
    const birthDate = new Date(dob);
    let age = today.getFullYear() - birthDate.getFullYear();
    const m = today.getMonth() - birthDate.getMonth();
    if (m < 0 || (m === 0 && today.getDate() < birthDate.getDate())) {
        age--;
    }
    return age;
  }
</script>

<div class="space-y-6">
  {#if loading}
    <div>Loading...</div>
  {:else if !patient}
    <div>Patient not found</div>
  {:else}
    <div class="flex items-center justify-between">
      <div class="space-y-1">
        <h1 class="text-3xl font-bold tracking-tight">{patient.last_name}, {patient.first_name}</h1>
        <div class="flex items-center gap-4 text-muted-foreground">
          <span>MRN: {patient.medical_record_number}</span>
          <span>DOB: {patient.date_of_birth} ({getAge(patient.date_of_birth)}yo)</span>
          <span>{patient.gender}</span>
        </div>
      </div>
      <div class="flex gap-2">
        <button class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2">
          Edit Profile
        </button>
        <button class="inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2">
          New Appointment
        </button>
      </div>
    </div>

    <div class="flex space-x-1 rounded-lg bg-muted p-1">
      {#each ['overview', 'clinical', 'appointments', 'billing'] as tab}
        <button
          on:click={() => activeTab = tab}
          class="inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 {activeTab === tab ? 'bg-background text-foreground shadow-sm' : 'hover:bg-background/50 text-muted-foreground'}"
        >
          {tab.charAt(0).toUpperCase() + tab.slice(1)}
        </button>
      {/each}
    </div>

    <div class="mt-6">
      {#if activeTab === 'overview'}
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          <div class="rounded-xl border bg-card text-card-foreground shadow p-6">
            <h3 class="font-semibold leading-none tracking-tight mb-4">Contact Information</h3>
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Phone</span>
                <span>{patient.phone}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Email</span>
                <span>{patient.email || '-'}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Address</span>
                <!-- In real data we'd look inside address object -->
                <span>123 Main St, Anytown</span> 
              </div>
            </div>
          </div>
          
          <div class="rounded-xl border bg-card text-card-foreground shadow p-6">
            <h3 class="font-semibold leading-none tracking-tight mb-4">Insurance</h3>
             <!-- Mock data -->
             <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Provider</span>
                <span>Blue Cross</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Member ID</span>
                <span>XYZ123456789</span>
              </div>
            </div>
          </div>

          <div class="rounded-xl border bg-card text-card-foreground shadow p-6">
            <h3 class="font-semibold leading-none tracking-tight mb-4">Emergency Contact</h3>
            <!-- Mock data -->
            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-muted-foreground">Name</span>
                <span>Jane Doe</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Relation</span>
                <span>Spouse</span>
              </div>
              <div class="flex justify-between">
                <span class="text-muted-foreground">Phone</span>
                <span>555-987-6543</span>
              </div>
            </div>
          </div>
        </div>
      {:else if activeTab === 'clinical'}
        <div class="grid gap-4 md:grid-cols-2">
            <div class="rounded-xl border bg-card text-card-foreground shadow p-6">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="font-semibold leading-none tracking-tight">Allergies</h3>
                    <button class="text-xs text-primary">Add</button>
                </div>
                {#if patient.allergies.length === 0}
                    <p class="text-sm text-muted-foreground">No known allergies</p>
                {:else}
                    <ul class="space-y-2">
                        {#each patient.allergies as allergy}
                            <li class="flex items-center justify-between text-sm rounded-md border p-2">
                                <span class="font-medium">{allergy.name}</span>
                                <span class="text-destructive text-xs px-2 py-0.5 rounded-full bg-destructive/10">{allergy.severity}</span>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>

            <div class="rounded-xl border bg-card text-card-foreground shadow p-6">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="font-semibold leading-none tracking-tight">Medications</h3>
                    <button class="text-xs text-primary">Add</button>
                </div>
                {#if patient.medications.length === 0}
                    <p class="text-sm text-muted-foreground">No active medications</p>
                {:else}
                    <ul class="space-y-2">
                        {#each patient.medications as med}
                            <li class="flex items-center justify-between text-sm rounded-md border p-2">
                                <div>
                                    <div class="font-medium">{med.name}</div>
                                    <div class="text-xs text-muted-foreground">{med.dosage} • {med.frequency}</div>
                                </div>
                                {#if med.active}
                                    <span class="text-green-600 text-xs px-2 py-0.5 rounded-full bg-green-100">Active</span>
                                {/if}
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>
      {:else}
         <div class="p-8 text-center text-muted-foreground bg-card rounded-xl border border-dashed">
            Feature coming soon
         </div>
      {/if}
    </div>
  {/if}
</div>
