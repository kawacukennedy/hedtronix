<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { FileText, Activity, Calendar, Pill } from 'lucide-svelte';
    // Badge removed as it was not found and not used yet

    export let items: Array<{
        id: string;
        type: 'NOTE' | 'VITAL' | 'APPOINTMENT' | 'MEDICATION';
        date: string;
        title: string;
        description?: string;
        meta?: any;
    }> = [];

    const icons = {
        NOTE: FileText,
        VITAL: Activity,
        APPOINTMENT: Calendar,
        MEDICATION: Pill
    };

    const colors = {
        NOTE: 'bg-blue-100 text-blue-800',
        VITAL: 'bg-red-100 text-red-800',
        APPOINTMENT: 'bg-purple-100 text-purple-800',
        MEDICATION: 'bg-green-100 text-green-800'
    };
</script>

<div class="space-y-8 relative before:absolute before:inset-0 before:ml-5 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-slate-300 before:to-transparent">
    {#each items as item}
        <div class="relative flex items-center justify-between md:justify-normal md:odd:flex-row-reverse group is-active">
            <!-- Icon -->
            <div class="flex items-center justify-center w-10 h-10 rounded-full border border-white bg-slate-50 shadow shrink-0 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2">
                <svelte:component this={icons[item.type]} class="w-5 h-5 text-slate-500" />
            </div>
            
            <!-- Cards -->
            <div class="w-[calc(100%-4rem)] md:w-[calc(50%-2.5rem)] p-4 rounded border border-slate-200 bg-white shadow-sm">
                <div class="flex items-center justify-between space-x-2 mb-1">
                    <div class="font-bold text-slate-900">{item.title}</div>
                    <time class="font-caveat font-medium text-amber-500 text-sm whitespace-nowrap">
                        {new Date(item.date).toLocaleDateString()}
                    </time>
                </div>
                <div class="text-slate-500 text-sm">
                    {item.description || ''}
                    {#if item.type === 'VITAL'}
                        <div class="mt-2 text-xs font-mono bg-slate-50 p-2 rounded">
                            BP: {item.meta.systolic}/{item.meta.diastolic} | HR: {item.meta.heartRate}
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    {/each}
    {#if items.length === 0}
        <div class="text-center text-muted-foreground py-8">No history events found.</div>
    {/if}
</div>
