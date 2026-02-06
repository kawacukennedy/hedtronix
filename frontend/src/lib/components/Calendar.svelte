<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Button } from "$lib/components/ui/button";
    import { ChevronLeft, ChevronRight, Calendar as CalIcon } from 'lucide-svelte';

    export let events: any[] = [];
    export let date = new Date();
    export let view: 'day' | 'week' | 'month' = 'day';

    const dispatch = createEventDispatcher();
    
    // Time slots handling for Day/Week view
    const START_HOUR = 8;
    const END_HOUR = 19; // 7 PM
    const HOURS = Array.from({length: END_HOUR - START_HOUR + 1}, (_, i) => i + START_HOUR);

    // Helpers
    $: currYear = date.getFullYear();
    $: currMonth = date.getMonth();
    
    // Month View Logic
    $: daysInMonth = new Date(currYear, currMonth + 1, 0).getDate();
    $: firstDayOfMonth = new Date(currYear, currMonth, 1).getDay(); // 0 = Sun
    $: monthDays = Array.from({length: daysInMonth}, (_, i) => i + 1);
    $: previousMonthDays = Array.from({length: firstDayOfMonth}, (_, i) => {
        const d = new Date(currYear, currMonth, 0);
        return d.getDate() - firstDayOfMonth + i + 1;
    });

    function isSameDay(d1: Date, d2: Date) {
        return d1.getFullYear() === d2.getFullYear() &&
               d1.getMonth() === d2.getMonth() &&
               d1.getDate() === d2.getDate();
    }

    function getEventsForDay(d: number) {
        const target = new Date(currYear, currMonth, d);
        return events.filter(e => {
            const date = new Date(e.startTime);
            return isSameDay(date, target);
        });
    }

    function handleEventClick(e: any) {
        dispatch('eventClick', e);
    }

    function handleDateClick(d: number) {
        const newDate = new Date(currYear, currMonth, d);
        dispatch('dateClick', newDate);
    }

    function changeDate(delta: number) {
        const newDate = new Date(date);
        if (view === 'month') {
            newDate.setMonth(newDate.getMonth() + delta);
        } else {
            newDate.setDate(newDate.getDate() + delta);
        }
        dispatch('change', newDate);
    }
</script>

<div class="flex flex-col h-full border rounded-lg bg-background shadow-sm overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b">
        <h2 class="text-xl font-semibold flex items-center gap-2">
            <CalIcon class="w-5 h-5 text-primary" />
            {#if view === 'month'}
                {date.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}
            {:else}
                {date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' })}
            {/if}
        </h2>
        <div class="flex items-center gap-2">
            <div class="flex border rounded-md overflow-hidden bg-muted">
                <button class={`px-3 py-1 text-sm ${view === 'day' ? 'bg-background shadow-sm font-medium' : 'text-muted-foreground'}`} on:click={() => dispatch('viewChange', 'day')}>Day</button>
                <button class={`px-3 py-1 text-sm ${view === 'month' ? 'bg-background shadow-sm font-medium' : 'text-muted-foreground'}`} on:click={() => dispatch('viewChange', 'month')}>Month</button>
            </div>
            <div class="flex items-center rounded-md border bg-background">
                <Button variant="ghost" size="icon" on:click={() => changeDate(-1)} class="rounded-none border-r">
                    <ChevronLeft class="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" on:click={() => changeDate(1)} class="rounded-none">
                    <ChevronRight class="h-4 w-4" />
                </Button>
            </div>
        </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-auto">
        {#if view === 'month'}
            <div class="grid grid-cols-7 h-full min-h-[500px]">
                <!-- Weekday Headers -->
                {#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
                    <div class="p-2 text-center text-sm font-medium text-muted-foreground border-b bg-muted/30">{day}</div>
                {/each}

                <!-- Previous Month Padding -->
                {#each previousMonthDays as day}
                    <div class="p-2 border-b border-r bg-muted/10 text-muted-foreground/50 min-h-[100px]">{day}</div>
                {/each}

                <!-- Current Month -->
                {#each monthDays as day}
                    <div 
                        class="p-1 border-b border-r min-h-[100px] relative hover:bg-muted/10 cursor-pointer transition-colors"
                        role="button"
                        tabindex="0"
                        on:click={() => handleDateClick(day)}
                        on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleDateClick(day)}
                    >
                        <div class={`text-sm font-medium h-7 w-7 flex items-center justify-center rounded-full ${isSameDay(new Date(currYear, currMonth, day), new Date()) ? 'bg-primary text-primary-foreground' : ''}`}>
                            {day}
                        </div>
                        <div class="mt-1 space-y-1">
                            {#each getEventsForDay(day).slice(0, 3) as evt}
                                <button class="block w-full text-left text-xs truncate px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 hover:bg-blue-200"
                                    on:click|stopPropagation={() => handleEventClick(evt)}>
                                    {new Date(evt.startTime).toLocaleTimeString([], {hour: 'numeric', minute:'2-digit'})} {evt.patientName || 'Visit'}
                                </button>
                            {/each}
                            {#if getEventsForDay(day).length > 3}
                                <div class="text-[10px] text-muted-foreground pl-1">
                                    + {getEventsForDay(day).length - 3} more
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {:else}
            <!-- Day View with Time Slots -->
            <div class="flex flex-col min-w-[600px]">
                 <div class="grid grid-cols-[60px_1fr] divide-y relative">
                    {#each HOURS as hour}
                        <div class="h-16 border-r text-xs text-muted-foreground p-2 text-right sticky left-0 bg-background z-10">
                            {hour > 12 ? hour - 12 : hour} {hour >= 12 ? 'PM' : 'AM'}
                        </div>
                        <div class="h-16 relative hover:bg-muted/5 group">
                            <!-- Events logic needed here to position absolutely -->
                            {#each events.filter(e => {
                                const d = new Date(e.startTime);
                                return isSameDay(d, date) && d.getHours() === hour;
                            }) as evt}
                                 {@const startMin = new Date(evt.startTime).getMinutes()}
                                 {@const duration = evt.duration || 30}
                                 <button class="absolute left-1 right-2 rounded border px-2 py-1 text-xs overflow-hidden z-20 bg-blue-100 border-blue-200 text-blue-800 hover:z-30 hover:shadow-md transition-all"
                                    style="top: {(startMin / 60) * 100}%; height: {(duration / 60) * 100}%;"
                                    on:click|stopPropagation={() => handleEventClick(evt)}>
                                     <div class="font-semibold">{evt.patientName}</div>
                                     <div>{evt.appointmentType}</div>
                                 </button>
                            {/each}
                            <!-- Time slot click -->
                             <button class="absolute inset-0 w-full h-full opacity-0 hover:opacity-100 flex items-center justify-center text-xs text-muted-foreground"
                                on:click={() => dispatch('slotClick', { date, hour })}>
                                <span class="group-hover:block hidden">+ Add</span>
                             </button>
                        </div>
                    {/each}
                 </div>
            </div>
        {/if}
    </div>
</div>
