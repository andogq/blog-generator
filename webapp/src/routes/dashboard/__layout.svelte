<script lang="ts">
    import { page } from "$app/stores";

    $: active = $page.url.pathname.replace(/^.+\/(.+)$/, "$1");
</script>

<div id="container">
    <div id="sidebar">
        {#each ["Domains", "Cache", "Account"] as link}
            <p class:active={active === link.toLowerCase()}><a class="no_link" href="/dashboard/{link.toLowerCase()}">{link}</a></p>
        {/each}
    </div>

    <div id="content">
        <slot />
    </div>
</div>

<style>
    #container {
        display: flex;
        flex-direction: row;
        align-items: flex-start;

        gap: var(--space-sm);

        margin: var(--space-sm) var(--space-md);

        overflow: hidden;
    }

    #container > * {
        padding: var(--space-sm);
        border-radius: var(--border-radius);
        background: var(--background-light);
    }

    #sidebar {
        display: flex;
        flex-direction: column;
        gap: var(--space-xs);

        flex-basis: 20%;
        flex-grow: 0;
        flex-shrink: 0;
    }

    #sidebar > * {
        font-size: 1rem;
        cursor: pointer;
        user-select: none;

        transition: color var(--transition);
    }

    #sidebar > .active, #sidebar > *:hover {
        color: var(--accent);
    }

    #content {
        flex-grow: 1;

        display: flex;
        flex-direction: column;
        gap: var(--space-xs);

        padding: var(--space-sm) var(--space-md);
    }
</style>

