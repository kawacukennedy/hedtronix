<script>
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  
  onMount(() => {
    if (!$auth.isAuthenticated) {
      goto('/login');
    }
  });
</script>

{#if $auth.isAuthenticated}
  <div class="min-h-screen bg-background">
    <header class="border-b bg-card">
      <div class="container flex h-16 items-center justify-between px-4">
        <div class="flex items-center gap-6">
          <a href="/" class="text-xl font-bold">HEDTRONIX</a>
          <nav class="flex gap-4 text-sm font-medium">
            <a href="/patients" class="text-muted-foreground transition-colors hover:text-foreground">Patients</a>
            <a href="/appointments" class="text-muted-foreground transition-colors hover:text-foreground">Schedule</a>
            <a href="/clinical-notes" class="text-muted-foreground transition-colors hover:text-foreground">Notes</a>
            <a href="/billing" class="text-muted-foreground transition-colors hover:text-foreground">Billing</a>
          </nav>
        </div>
        <div class="flex items-center gap-4">
          <span class="text-sm text-muted-foreground">{$auth.user?.name}</span>
          <div class="h-8 w-8 rounded-full bg-primary/20"></div>
        </div>
      </div>
    </header>
    <main class="container py-6 px-4">
      <slot />
    </main>
  </div>
{/if}
