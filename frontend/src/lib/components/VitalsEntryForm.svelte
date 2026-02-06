<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Plus } from 'lucide-svelte';

    const dispatch = createEventDispatcher();

    let vitals = {
        systolic: '',
        diastolic: '',
        heartRate: '',
        temperature: '',
        spO2: '',
        respiratoryRate: ''
    };

    function handleSubmit() {
        if (!vitals.systolic || !vitals.diastolic) return; // minimal validation

        dispatch('submit', {
            ...vitals,
            recordedAt: new Date().toISOString(),
            // Convert to numbers
            systolic: Number(vitals.systolic),
            diastolic: Number(vitals.diastolic),
            heartRate: Number(vitals.heartRate),
            temperature: Number(vitals.temperature),
            spO2: Number(vitals.spO2),
            respiratoryRate: Number(vitals.respiratoryRate)
        });
        
        // Reset form
        vitals = {
            systolic: '',
            diastolic: '',
            heartRate: '',
            temperature: '',
            spO2: '',
            respiratoryRate: ''
        };
    }
</script>

<div class="grid gap-4 p-4 border rounded-lg bg-card text-card-foreground">
    <div class="flex items-center justify-between">
        <h3 class="font-semibold">Record Vitals</h3>
    </div>
    <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
        <div class="space-y-2">
            <Label>BP (mmHg)</Label>
            <div class="flex items-center gap-2">
                <Input type="number" placeholder="Sys" bind:value={vitals.systolic} class="w-20" />
                <span>/</span>
                <Input type="number" placeholder="Dia" bind:value={vitals.diastolic} class="w-20" />
            </div>
        </div>
        <div class="space-y-2">
            <Label>Heart Rate (bpm)</Label>
            <Input type="number" bind:value={vitals.heartRate} />
        </div>
        <div class="space-y-2">
            <Label>Temp (°F)</Label>
            <Input type="number" step="0.1" bind:value={vitals.temperature} />
        </div>
        <div class="space-y-2">
            <Label>SpO2 (%)</Label>
            <Input type="number" bind:value={vitals.spO2} />
        </div>
        <div class="space-y-2">
            <Label>Resp. Rate (bpm)</Label>
            <Input type="number" bind:value={vitals.respiratoryRate} />
        </div>
        <div class="flex items-end">
            <Button on:click={handleSubmit} class="w-full">
                <Plus class="mr-2 h-4 w-4" /> Add Entry
            </Button>
        </div>
    </div>
</div>
