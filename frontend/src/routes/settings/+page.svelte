<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import { syncStore } from '$lib/stores/sync';
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Smartphone, RefreshCw, Trash2, LogOut } from 'lucide-svelte';

    async function forceSync() {
        await syncStore.sync();
    }

    function clearData() {
        if (confirm('WARNING: This will delete all local data. Only do this if you are sure all changes have synced. Continue?')) {
            // Logic to clear IDB - simplistic approach for now
            // In reality, we'd need a method on db class
            indexedDB.deleteDatabase('hedtronix-db');
            window.location.reload();
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-3xl mx-auto">
    <div>
        <h1 class="text-3xl font-bold tracking-tight">Settings</h1>
        <p class="text-muted-foreground">System and device configuration</p>
    </div>

    <Card.Root>
        <Card.Header>
            <Card.Title>Device Registration</Card.Title>
        </Card.Header>
        <Card.Content class="space-y-4">
            <div class="flex items-center justify-between p-4 border rounded-lg bg-muted/20">
                <div class="flex items-center gap-4">
                    <div class="p-2 bg-primary/10 rounded-full">
                        <Smartphone class="h-6 w-6 text-primary" />
                    </div>
                    <div>
                        <div class="font-medium">Current Device</div>
                        <div class="text-sm text-muted-foreground font-mono">{$auth.deviceId}</div>
                    </div>
                </div>
                <div class="text-xs px-2 py-1 bg-green-100 text-green-800 rounded">
                    Active
                </div>
            </div>
        </Card.Content>
    </Card.Root>

    <Card.Root>
        <Card.Header>
            <Card.Title>Synchronization</Card.Title>
            <Card.Description>Manage offline data and sync status</Card.Description>
        </Card.Header>
        <Card.Content class="space-y-4">
             <div class="grid gap-4 md:grid-cols-2">
                 <div class="p-4 border rounded-lg">
                     <div class="text-sm text-muted-foreground">Status</div>
                     <div class="text-lg font-bold flex items-center gap-2">
                         {$syncStore.status}
                     </div>
                 </div>
                 <div class="p-4 border rounded-lg">
                     <div class="text-sm text-muted-foreground">Pending Changes</div>
                     <div class="text-lg font-bold">{$syncStore.pendingCount}</div>
                 </div>
             </div>
             
             <div class="flex gap-4">
                 <Button on:click={forceSync} disabled={$syncStore.status === 'SYNCING'}>
                     <RefreshCw class={`mr-2 h-4 w-4 ${$syncStore.status === 'SYNCING' ? 'animate-spin' : ''}`} />
                     Sync Now
                 </Button>
                 
                 <Button variant="destructive" on:click={clearData}>
                     <Trash2 class="mr-2 h-4 w-4" />
                     Clear Local Data
                 </Button>
             </div>
        </Card.Content>
    </Card.Root>
    
    <div class="flex justify-center">
        <Button variant="ghost" class="text-red-600 hover:text-red-700 hover:bg-red-50" on:click={() => auth.logout()}>
            <LogOut class="mr-2 h-4 w-4" /> Sign Out
        </Button>
    </div>
</div>
