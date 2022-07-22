<script lang="ts">
    export let name: string;
</script>

<div class="card">
    <h3>{name}</h3>

    <slot />
</div>

<style>
    .card {
        background: var(--background-light);
        border-radius: var(--border-radius);

        padding: var(--space-sm) var(--space-md);
        margin: var(--space-sm) var(--space-lg);

        display: flex;
        flex-direction: column;
        gap: var(--space-xs);
    }
</style>
