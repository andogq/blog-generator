<script lang="ts">
    import { onMount } from "svelte";

    let div: HTMLDivElement;

    onMount(() => {
        let portal = document.createElement("div");
        portal.appendChild(div);

        document.body.appendChild(portal);

        return () => document.body.removeChild(portal);
    });
</script>

<div bind:this={div}>
 <slot />
</div>
