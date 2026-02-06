<script lang="ts">
    import { onMount } from 'svelte';
    import { db } from '$lib/db/indexed-db';
    import { Button } from "$lib/components/ui/button";
    import * as Card from "$lib/components/ui/card";
    import { Plus, Download, FileText } from 'lucide-svelte';

    let invoices: any[] = [];
    let loading = true;

    onMount(async () => {
        try {
            const allInvoices = await db.getBillingEntries();
            const allPatients = await db.getAllPatients();
            const patientMap = new Map(allPatients.map(p => [p.id, `${p.firstName} ${p.lastName}`]));

            invoices = allInvoices.map((inv: any) => ({
                ...inv,
                patientName: patientMap.get(inv.patientId) || 'Unknown'
            })).sort((a: any, b: any) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
        } catch (e) {
            console.error(e);
        } finally {
            loading = false;
        }
    });

    const statusColors: any = {
        'DRAFT': 'bg-gray-100 text-gray-800',
        'SENT': 'bg-blue-100 text-blue-800',
        'PAID': 'bg-green-100 text-green-800',
        'OVERDUE': 'bg-red-100 text-red-800',
        'SUBMITTED': 'bg-purple-100 text-purple-800',
        'REJECTED': 'bg-red-100 text-red-800',
        'APPROVED': 'bg-green-100 text-green-800'
    };

    import { Tabs, TabsContent, TabsList, TabsTrigger } from "$lib/components/ui/tabs";
</script>

<div class="h-[calc(100vh-6rem)] flex flex-col p-6 space-y-6">
    <div class="flex items-center justify-between shrink-0">
        <div>
            <h1 class="text-3xl font-bold tracking-tight">Billing & Insurance</h1>
            <p class="text-muted-foreground">Manage invoices and insurance claims.</p>
        </div>
        <div class="flex gap-2">
            <Button href="/billing/new">
                <Plus class="mr-2 h-4 w-4" /> New Invoice
            </Button>
        </div>
    </div>

    <!-- Metrics Cards (Keep existing) -->
    <div class="grid gap-4 grid-cols-1 md:grid-cols-3">
        <!-- ... (Keep cards as is) ... -->
        <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Total Revenue</Card.Title>
                <div class="text-muted-foreground">$</div>
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">$12,345.00</div>
                <p class="text-xs text-muted-foreground">+20.1% from last month</p>
            </Card.Content>
        </Card.Root>
         <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Outstanding</Card.Title>
                 <div class="text-muted-foreground">!</div>
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">$2,350.00</div>
                <p class="text-xs text-muted-foreground">8 invoices overdue</p>
            </Card.Content>
        </Card.Root>
         <Card.Root>
            <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
                <Card.Title class="text-sm font-medium">Claims Pending</Card.Title>
                <FileText class="h-4 w-4 text-muted-foreground" />
            </Card.Header>
            <Card.Content>
                <div class="text-2xl font-bold">12</div>
                <p class="text-xs text-muted-foreground">Processing with insurance</p>
            </Card.Content>
        </Card.Root>
    </div>

    <Tabs defaultValue="invoices" class="flex-1 flex flex-col">
        <div class="flex items-center justify-between mb-4">
            <TabsList>
                <TabsTrigger value="invoices">Invoices</TabsTrigger>
                <TabsTrigger value="claims">Insurance Claims</TabsTrigger>
            </TabsList>
        </div>

        <TabsContent value="invoices" class="flex-1 mt-0">
            <div class="rounded-md border bg-card">
                 <!-- Existing Invoice Table -->
                <div class="p-4 border-b">
                    <h3 class="font-semibold">Recent Invoices</h3>
                </div>
                {#if loading}
                    <div class="p-8 text-center text-muted-foreground animate-pulse">Loading invoices...</div>
                {:else if invoices.length === 0}
                    <div class="p-12 text-center text-muted-foreground">No invoices found</div>
                {:else}
                <div class="relative w-full overflow-auto">
                    <table class="w-full caption-bottom text-sm">
                        <thead class="[&_tr]:border-b">
                            <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Invoice #</th>
                                <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Date</th>
                                <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Patient</th>
                                <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Amount</th>
                                <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Status</th>
                                <th class="h-12 px-4 text-right align-middle font-medium text-muted-foreground">Actions</th>
                            </tr>
                        </thead>
                        <tbody class="[&_tr:last-child]:border-0">
                            {#each invoices as inv}
                                <tr class="border-b transition-colors hover:bg-muted/50">
                                    <td class="p-4 font-medium">{inv.id.slice(0,8).toUpperCase()}</td>
                                    <td class="p-4">{new Date(inv.createdAt).toLocaleDateString()}</td>
                                    <td class="p-4">{inv.patientName}</td>
                                    <td class="p-4">${inv.totalAmount.toFixed(2)}</td>
                                    <td class="p-4">
                                        <span class={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent ${statusColors[inv.status] || 'bg-gray-100'}`}>
                                            {inv.status}
                                        </span>
                                    </td>
                                    <td class="p-4 text-right">
                                        <Button variant="ghost" size="icon">
                                            <Download class="h-4 w-4" />
                                        </Button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
                {/if}
            </div>
        </TabsContent>

        <TabsContent value="claims" class="flex-1 mt-0">
            <Card.Root>
                <Card.Header>
                    <Card.Title>Insurance Claims</Card.Title>
                    <Card.Description>Manage claims submitted to insurance providers.</Card.Description>
                </Card.Header>
                <Card.Content>
                    <!-- Mock Claims Data for MVP -->
                    <div class="relative w-full overflow-auto">
                        <table class="w-full caption-bottom text-sm">
                            <thead class="[&_tr]:border-b">
                                <tr class="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                    <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Claim ID</th>
                                    <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Patient</th>
                                    <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Provider</th>
                                    <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Date Submitted</th>
                                    <th class="h-12 px-4 text-left align-middle font-medium text-muted-foreground">Status</th>
                                </tr>
                            </thead>
                            <tbody class="[&_tr:last-child]:border-0">
                                <tr class="border-b transition-colors hover:bg-muted/50">
                                    <td class="p-4 font-medium">CLM-001</td>
                                    <td class="p-4">John Doe</td>
                                    <td class="p-4">BlueCross</td>
                                    <td class="p-4">2023-10-25</td>
                                    <td class="p-4"><span class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-purple-100 text-purple-800">SUBMITTED</span></td>
                                </tr>
                                <tr class="border-b transition-colors hover:bg-muted/50">
                                    <td class="p-4 font-medium">CLM-002</td>
                                    <td class="p-4">Jane Smith</td>
                                    <td class="p-4">Aetna</td>
                                    <td class="p-4">2023-10-24</td>
                                    <td class="p-4"><span class="inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-green-100 text-green-800">APPROVED</span></td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </Card.Content>
            </Card.Root>
        </TabsContent>
    </Tabs>
</div>
