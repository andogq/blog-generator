<script lang="ts">
    export let options: {
        name: string,
        description: string,
        button_text: string,
        on_click: () => void
    }[];
</script>

{#each options as option}
    <div class="option">
        <h3 class="no_line">{option.name}</h3>
        <p>{option.description}</p>
        <button on:click={option.on_click}>{option.button_text}</button>
    </div>
{/each}

<style>
    .option {
        display: grid;
        grid-template: auto auto / 1fr auto;
        gap: var(--space-xs);

        align-items: center;
    }

    .option > h3 {
        grid-area: 1 / 1 / 2 / 2;
    }

    .option > p {
        grid-area: 2 / 1 / 3 / 2;
    }

    .option > button {
        grid-area: 1 / 2 / 3 / 3;
    }
</style>
