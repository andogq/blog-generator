<script lang="ts">
    import Scroll from "$lib/components/Scroll.svelte";

    import Domains from "$lib/components/dashboard/Domains.svelte";
    import Cache from "$lib/components/dashboard/Cache.svelte";
    import Account from "$lib/components/dashboard/Account.svelte";

    let pages = ["Domains", "Cache", "Account"];
    let items = [Domains, Cache, Account];
    let active = 0;

    function change_page(i: number) {
        active = i;
    }
</script>

<div id="container">
    <div id="sidebar">
        {#each pages as link, i}
            <p on:click={change_page.bind(undefined, i)} class:active={pages[active] === link}>{link}</p>
        {/each}
    </div>

    <Scroll
        bind:step={active}
        {items}
    />
</div>

<style>
    #container {
        display: flex;
        flex-direction: row;
        align-items: flex-start;

        gap: var(--space-sm);

        height: 100%;

        overflow: hidden;
    }

    #sidebar {
        display: flex;
        flex-direction: column;
        gap: var(--space-xs);

        margin: var(--space-sm);
        padding: var(--space-sm);
        border-radius: var(--border-radius);
        background: var(--background-light);

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

</style>

