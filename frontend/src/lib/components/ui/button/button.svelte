<script lang="ts">
  import { type ButtonVariant, buttonVariants } from "./module";
  import { cn } from "$lib/utils";

  export let variant: ButtonVariant = "default";
  export let size: "default" | "sm" | "lg" | "icon" = "default";
  export let href: string | undefined = undefined;
  let className: string | undefined = undefined;
  export { className as class };
</script>

{#if href}
  <a href={href} class={cn(buttonVariants({ variant, size }), className)} {...$$restProps}>
    <slot />
  </a>
{:else}
  <button class={cn(buttonVariants({ variant, size }), className)} on:click {...$$restProps}>
    <slot />
  </button>
{/if}
