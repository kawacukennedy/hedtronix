<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { syncStore } from '$lib/stores/sync';
    import { goto } from '$app/navigation';
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import * as Card from "$lib/components/ui/card";
    import { ArrowLeft, Trash2, Plus } from 'lucide-svelte';

    let loading = false;
    let patients: any[] = [];
    
    let invoice = {
        patientId: '',
        status: 'DRAFT',
        items: [
            { description: 'Consultation Fee', quantity: 1, unitPrice: 150.00 }
        ],
        notes: '',
        dueDate: new Date(Date.now() + 30*24*60*60*1000).toISOString().split('T')[0] // Net 30 default
    };

    $: subtotal = invoice.items.reduce((sum, item) => sum + (item.quantity * item.unitPrice), 0);
    $: tax = 0; // standard for medical? often 0
    $: total = subtotal + tax;

    onMount(async () => {
        patients = await db.getAllPatients();
    });

    function addItem() {
        invoice.items = [...invoice.items, { description: '', quantity: 1, unitPrice: 0 }];
    }

    function removeItem(index: number) {
        invoice.items = invoice.items.filter((_, i) => i !== index);
    }

    async function handleSave() {
        loading = true;
        try {
            await syncStore.returnChange('Invoice', crypto.randomUUID(), 'CREATE', {
                ...invoice,
                totalAmount: total,
                createdAt: new Date().toISOString()
            });
            goto('/billing');
        } catch (e) {
            console.error(e);
            alert('Failed to create invoice');
        } finally {
            loading = false;
        }
    }
</script>

<div class="flex flex-col gap-6 p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4">
        <Button variant="ghost" size="icon" href="/billing">
            <ArrowLeft class="h-4 w-4" />
        </Button>
        <div>
            <h1 class="text-3xl font-bold tracking-tight">New Invoice</h1>
            <p class="text-muted-foreground">Create a new bill for patient</p>
        </div>
    </div>

    <div class="grid gap-6 md:grid-cols-3">
        <div class="md:col-span-2 space-y-6">
            <Card.Root>
                <Card.Content class="space-y-6 pt-6">
                    <div class="grid gap-4 md:grid-cols-2">
                        <div class="space-y-2">
                            <Label for="patient">Patient</Label>
                            <select id="patient" class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2" bind:value={invoice.patientId}>
                                <option value="">Select Patient...</option>
                                {#each patients as p}
                                    <option value={p.id}>{p.lastName}, {p.firstName}</option>
                                {/each}
                            </select>
                        </div>
                        <div class="space-y-2">
                            <Label for="dueDate">Due Date</Label>
                            <Input type="date" id="dueDate" bind:value={invoice.dueDate} />
                        </div>
                    </div>

                    <!-- Line Items -->
                     <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <h3 class="font-medium">Line Items</h3>
                            <Button variant="outline" size="sm" on:click={addItem}>
                                <Plus class="h-4 w-4 mr-2" /> Add Item
                            </Button>
                        </div>
                        
                        <div class="space-y-2">
                             {#each invoice.items as item, i}
                                <div class="flex gap-2 items-start">
                                    <div class="flex-1">
                                        <Input placeholder="Description" bind:value={item.description} />
                                    </div>
                                    <div class="w-20">
                                        <Input type="number" placeholder="Qty" min="1" bind:value={item.quantity} />
                                    </div>
                                    <div class="w-32">
                                        <Input type="number" placeholder="Price" min="0" step="0.01" bind:value={item.unitPrice} />
                                    </div>
                                    <Button variant="ghost" size="icon" on:click={() => removeItem(i)} disabled={invoice.items.length === 1}>
                                        <Trash2 class="h-4 w-4 text-red-500" />
                                    </Button>
                                </div>
                             {/each}
                        </div>
                     </div>
                </Card.Content>
            </Card.Root>
            
            <Card.Root>
                <Card.Content class="pt-6">
                    <Label for="notes">Notes</Label>
                    <textarea 
                        id="notes" 
                        class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm min-h-[100px] mt-2"
                        placeholder="Payment terms, instructions, etc."
                        bind:value={invoice.notes}
                    ></textarea>
                </Card.Content>
            </Card.Root>
        </div>

        <div class="space-y-6">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Summary</Card.Title>
                </Card.Header>
                <Card.Content class="space-y-4">
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Subtotal</span>
                        <span>${subtotal.toFixed(2)}</span>
                    </div>
                    <div class="flex justify-between text-sm">
                        <span class="text-muted-foreground">Tax (0%)</span>
                        <span>$0.00</span>
                    </div>
                    <div class="border-t pt-4 flex justify-between font-bold">
                        <span>Total</span>
                        <span>${total.toFixed(2)}</span>
                    </div>
                </Card.Content>
                <Card.Footer>
                     <Button class="w-full" on:click={handleSave} disabled={loading || !invoice.patientId}>
                        {#if loading}Creating...{:else}Create Invoice{/if}
                     </Button>
                </Card.Footer>
            </Card.Root>
        </div>
    </div>
</div>
