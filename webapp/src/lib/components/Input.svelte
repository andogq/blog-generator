<script lang="ts">
    export let label: string;
    export let placeholder: string;

    export let value: string;
</script>

<div id="input">
    <label for="input_{label}">{label}</label>
    <input
        id="input_{label}"
        type="text"
        {placeholder}
        bind:value
    />
</div>

<style>
    #input {
        display: flex;
        flex-direction: column;
        
        padding: var(--space-xs);

        border: var(--border-width) solid var(--text);
        border-radius: var(--border-radius);

        transition: border var(--transition);
    }

    #input:focus-within {
        border-color: var(--accent);
    }

    label {
        font-size: 0.75rem;
        user-select: none;
    }

    input {
        background: none;
        color: inherit;
        border: none;

        font-size: 1rem;
    }

    input:focus {
        outline: none;
    }
</style>

