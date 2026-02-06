<script lang="ts">
  import { Tabs as TabsPrimitive } from "bits-ui";
  import { cn } from "$lib/utils";

  let className: string | undefined = undefined;
  export { className as class };
  export let value: string | undefined = undefined;
  export let onValueChange: ((value: string) => void) | undefined = undefined;
</script>

<TabsPrimitive.Root
  bind:value
  {onValueChange}
  class={cn(className)}
  {...$$restProps}
>
  <slot />
</TabsPrimitive.Root>
