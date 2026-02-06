<script lang="ts">
    export let data: { label: string; value: number }[] = [];
    export let height = 300;
    export let width = 600;
    export let color = '#3b82f6';

    $: maxValue = Math.max(...data.map(d => d.value), 0) || 100;
    $: barWidth = (width - 60) / data.length; // 60px padding
</script>

<svg {width} {height} viewBox="0 0 {width} {height}" class="overflow-visible">
    <!-- Y Axis -->
    <line x1="40" y1="20" x2="40" y2={height - 20} stroke="#94a3b8" />
    
    <!-- X Axis -->
    <line x1="40" y1={height - 20} x2={width} y2={height - 20} stroke="#94a3b8" />

    {#each data as item, i}
        {@const barHeight = (item.value / maxValue) * (height - 40)}
        {@const x = 40 + (i * barWidth) + (barWidth * 0.1)}
        {@const y = height - 20 - barHeight}
        
        <g class="group">
            <rect
                x={x}
                y={y}
                width={barWidth * 0.8}
                height={barHeight}
                fill={color}
                rx="4"
                class="transition-all hover:opacity-80"
            />
            <!-- Tooltip / Label -->
            <text x={x + (barWidth * 0.4)} y={height - 5} text-anchor="middle" class="text-[10px] fill-gray-500">
                {item.label}
            </text>
            <text x={x + (barWidth * 0.4)} y={y - 5} text-anchor="middle" class="text-xs font-bold fill-gray-700 opacity-0 group-hover:opacity-100 transition-opacity">
                {item.value}
            </text>
        </g>
    {/each}
</svg>
