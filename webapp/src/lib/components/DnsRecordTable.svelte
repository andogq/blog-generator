<script lang="ts">
    import { createEventDispatcher } from "svelte";

    type DnsRecord = {
        record_type: string,
        name: string,
        value: string
    }

    export let records: DnsRecord[];
    export let loading: boolean = false;
    export let disable_refresh = false;

    $: show_message = loading || records.length === 0;

    let dispatch = createEventDispatcher();

    function trigger_refresh() {
        dispatch("refresh");
    }

    function copy(e: MouseEvent) {
        if (e.target instanceof HTMLElement) {
            navigator.clipboard.writeText(e.target.innerText)
                .then(() => {

                })
                .catch(e => console.error(e));
        }
    }
</script>

<div id="container">
    {#if !show_message}
        <table>
            <thead>
                <tr>
                    <th style="width: 25%">Record Type</th>
                    <th>Record Name</th>
                    <th>Record Content</th>
                </tr>
            </thead>
            <tbody>
                {#each records as record}
                    <tr>
                        <td>
                            <div>
                                {record.record_type}
                            </div>
                        </td>
                        <td>
                            <div on:click={copy} class="copy">
                                {record.name}
                            </div>
                        </td>
                        <td>
                            <div on:click={copy} class="copy">
                                {record.value}
                            </div>
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {:else}
        <div id="message">
            <p>
                {#if loading}
                    Loading records...
                {:else if records.length === 0}
                    There are no records to show!
                {/if}
            </p>
        </div>
    {/if}

    <div id="toolbar">
        {#if !show_message}
            <p>Click to copy a record name or value</p>
        {/if}

        {#if !disable_refresh}
            <button on:click={trigger_refresh} disabled={loading}>Refresh</button>
        {/if}
    </div>
</div>

<style>
    #toolbar {
        margin-top: 0.5rem;

        display: flex;
        flex-direction: row;
        justify-content: flex-end;
        align-items: center;
        gap: 1rem;
    }

    table {
        width: 100%;
        table-layout: fixed;
        border-collapse: collapse;
    }

    th, td {
        padding: 0.5rem 1rem;
        text-align: left;
    }

    td > div {
        white-space: nowrap;
        overflow-x: scroll;
        font-family: monospace;
    }

    thead {
        border-bottom: var(--border-width) solid var(--accent);
    }

    tr:not(:last-child) {
        border-bottom: var(--border-width) dotted var(--accent);
    }

    .copy {
        cursor: pointer;
        transition: transform var(--transition);
    }

    .copy:active {
        transform: scale(95%);
    }

    #message {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2rem 0;

        color: var(--grey);
    }
</style>

