<script>
  import '../app.css';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth';
  import { syncStore } from '$lib/stores/sync';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { Wifi, WifiOff, RefreshCw, AlertCircle, Menu, X } from 'lucide-svelte';
  import { Button } from "$lib/components/ui/button";
  
  // Exports to satisfy SvelteKit prop passing without warnings
  export let data;
  export let params = undefined;
  
  // Silence unused warnings
  $: ({ data, params }); 

  let isMobileMenuOpen = false;

  function toggleMobileMenu() {
    isMobileMenuOpen = !isMobileMenuOpen;
  }

  function closeMobileMenu() {
    isMobileMenuOpen = false;
  }
  
  onMount(() => {
    // BYPASS AUTH FOR TESTING
    // if (!$auth.isAuthenticated && $page.url.pathname !== '/login') {
    //   goto('/login');
    // } else 
    
    if ($auth.isAuthenticated) {
      syncStore.init();
    }
  });
</script>

<div class="min-h-screen bg-background">
  {#if $auth.isAuthenticated}
    <header class="border-b bg-card sticky top-0 z-50">
      <div class="container flex h-16 items-center justify-between px-4">
        <div class="flex items-center gap-6">
          <Button variant="ghost" size="icon" class="md:hidden" on:click={toggleMobileMenu}>
            <Menu class="h-5 w-5" />
          </Button>
          <a href="/" class="text-xl font-bold tracking-tight text-primary">HEDTRONIX</a>
          <nav class="hidden md:flex gap-4 text-sm font-medium">
            <a href="/patients" class="text-muted-foreground transition-colors hover:text-foreground">Patients</a>
            <a href="/appointments" class="text-muted-foreground transition-colors hover:text-foreground">Schedule</a>
            <a href="/notes" class="text-muted-foreground transition-colors hover:text-foreground">Notes</a>
            <a href="/billing" class="text-muted-foreground transition-colors hover:text-foreground">Billing</a>
            <a href="/analytics" class="text-muted-foreground transition-colors hover:text-foreground">Analytics</a>
          </nav>
        </div>
        <div class="flex items-center gap-4">
          <!-- Sync Status Indicator -->
          <div class="flex items-center gap-2 text-xs font-medium px-3 py-1.5 rounded-full bg-secondary/50 border">
              {#if $syncStore.status === 'ONLINE'}
                  <Wifi class="h-3.5 w-3.5 text-green-600" />
                  <span class="text-green-700 hidden md:inline">Online</span>
              {:else if $syncStore.status === 'SYNCING'}
                  <RefreshCw class="h-3.5 w-3.5 text-blue-600 animate-spin" />
                  <span class="text-blue-700 hidden md:inline">Syncing...</span>
              {:else if $syncStore.status === 'OFFLINE'}
                  <WifiOff class="h-3.5 w-3.5 text-amber-600" />
                  <span class="text-amber-700 hidden md:inline">Offline</span>
                  {#if $syncStore.pendingCount > 0}
                    <span class="bg-amber-200 text-amber-800 text-[10px] px-1.5 rounded-full">{$syncStore.pendingCount}</span>
                  {/if}
              {:else}
                  <AlertCircle class="h-3.5 w-3.5 text-red-600" />
                  <span class="text-red-700 hidden md:inline">Error</span>
              {/if}
          </div>

          <div class="flex items-center gap-2 border-l pl-4">
            <span class="text-sm font-medium">{$auth.user?.name}</span>
            <div class="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold">
                {$auth.user?.name?.[0] || 'U'}
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Mobile Menu -->
    {#if isMobileMenuOpen}
      <button 
        class="md:hidden fixed inset-0 z-40 bg-background/80 backdrop-blur-sm w-full h-full cursor-default border-none" 
        on:click={closeMobileMenu} 
        on:keydown={(e) => e.key === 'Escape' && closeMobileMenu()}
        aria-label="Close menu"
        type="button"
      ></button>
      <div class="md:hidden fixed inset-y-0 left-0 z-50 w-3/4 max-w-sm bg-card border-r shadow-lg p-6 transition-transform duration-300 ease-in-out">
         <div class="flex items-center justify-between mb-8">
            <span class="text-xl font-bold tracking-tight text-primary">HEDTRONIX</span>
            <Button variant="ghost" size="icon" on:click={closeMobileMenu}>
              <X class="h-5 w-5" />
            </Button>
         </div>
         <nav class="flex flex-col gap-4 text-lg font-medium">
            <a href="/patients" class="text-muted-foreground hover:text-foreground" on:click={closeMobileMenu}>Patients</a>
            <a href="/appointments" class="text-muted-foreground hover:text-foreground" on:click={closeMobileMenu}>Schedule</a>
            <a href="/notes" class="text-muted-foreground hover:text-foreground" on:click={closeMobileMenu}>Notes</a>
            <a href="/billing" class="text-muted-foreground hover:text-foreground" on:click={closeMobileMenu}>Billing</a>
            <a href="/analytics" class="text-muted-foreground hover:text-foreground" on:click={closeMobileMenu}>Analytics</a>
         </nav>
      </div>
    {/if}
    <main class="container py-6 px-4">
      <slot />
    </main>
  {:else}
    <slot />
  {/if}
</div>
