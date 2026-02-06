<script lang="ts">
    export let data: any[] = [];
    export let width = 600;
    export let height = 300;
    export let keys = ['systolic', 'diastolic'];
    export let colors = ['#ef4444', '#3b82f6']; // red-500, blue-500

    $: sortedData = [...data].sort((a, b) => new Date(a.recordedAt).getTime() - new Date(b.recordedAt).getTime());
    
    $: padding = 40;
    $: chartWidth = width - padding * 2;
    $: chartHeight = height - padding * 2;

    $: x = (d: any) => {
        if (sortedData.length < 2) return chartWidth / 2;
        const minTime = new Date(sortedData[0].recordedAt).getTime();
        const maxTime = new Date(sortedData[sortedData.length - 1].recordedAt).getTime();
        const time = new Date(d.recordedAt).getTime();
        return (time - minTime) / (maxTime - minTime) * chartWidth;
    };

    $: y = (val: number) => {
        // Find min/max for all keys to set scale
        let min = Infinity;
        let max = -Infinity;
        sortedData.forEach(d => {
            keys.forEach(k => {
                if (d[k] < min) min = d[k];
                if (d[k] > max) max = d[k];
            });
        });
        
        // Add buffer
        const range = max - min || 10;
        min -= range * 0.1;
        max += range * 0.1;
        
        if (min === Infinity) return chartHeight / 2;

        return chartHeight - ((val - min) / (max - min) * chartHeight);
    };

    $: paths = keys.map((key, i) => {
        if (sortedData.length === 0) return '';
        const d = sortedData.map((pt, index) => {
            const xPos = x(pt);
            const yPos = y(pt[key]);
            return `${index === 0 ? 'M' : 'L'} ${xPos} ${yPos}`;
        }).join(' ');
        return { d, color: colors[i] };
    });
</script>

<div class="w-full overflow-x-auto">
    <svg {width} {height} class="bg-card border rounded-lg">
        <g transform={`translate(${padding}, ${padding})`}>
            <!-- Grid Lines (Horizontal) -->
            {#each [0, 0.25, 0.5, 0.75, 1] as t}
                <line x1="0" y1={chartHeight * t} x2={chartWidth} y2={chartHeight * t} stroke="#e5e7eb" stroke-dasharray="4" />
            {/each}

            <!-- Lines -->
            {#each paths as path}
                <path d={path.d} fill="none" stroke={path.color} stroke-width="2" />
                
                <!-- Points -->
                {#each sortedData as pt}
                     <!-- Could add tooltips here -->
                    <circle cx={x(pt)} cy={y(pt.systolic)} r="4" fill="white" stroke={colors[0]} stroke-width="2" />
                    <circle cx={x(pt)} cy={y(pt.diastolic)} r="4" fill="white" stroke={colors[1]} stroke-width="2" />
                {/each}
            {/each}
            
            <!-- Axes Labels -->
             <text x={-padding + 10} y={chartHeight / 2} transform="rotate(-90, -30, 150)" class="text-xs fill-muted-foreground">BP (mmHg)</text>
             <text x={chartWidth / 2} y={chartHeight + 20} class="text-xs fill-muted-foreground text-center">Time</text>

        </g>
    </svg>
</div>
