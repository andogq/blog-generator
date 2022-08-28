<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { fade } from "svelte/transition";

    export let title: string;
    export let open: boolean;
    export let width: string = "auto";

    let dispatch = createEventDispatcher();

    function on_close() {
        dispatch("close");
    }
</script>

{#if open}
    <div
        id="shadow"
        on:click|self={on_close}
        transition:fade={{ duration: 100 }}
    >
        <div id="container" style:width>
            <h1 id="title">{title}</h1>

            <div id="content">
                <slot />
            </div>

            <div id="buttons">
                <slot name="buttons">
                <button on:click={on_close}>Close</button>
                </slot>
            </div>
        </div>
    </div>
{/if}

<style>
    #shadow {
        position: fixed;
        top: 0;
        left: 0;
        height: 100vh;
        width: 100vw;

        background: rgba(0, 0, 0, 0.7);

        display: flex;
        align-items: center;
        justify-content: center;
    }

    #container {
        padding: 2rem;
        background: var(--background);
        border-radius: var(--border-radius);

        max-height: 90vh;

        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    #content {
        margin: 0 1rem;

        overflow-y: scroll;
        overflow-x: hidden;

        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    #buttons {
        display: flex;
        flex-direction: row;
        justify-content: flex-end;
        gap: 1rem;
    }
</style>
