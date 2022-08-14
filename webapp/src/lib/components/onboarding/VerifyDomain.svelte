<script lang="ts">
    import Card from "./Card.svelte";
    import onboarding from "$lib/stores/onboarding";
    import { goto } from "$app/navigation";

    let loading = false;

    let timeout = 0;
    let interval: NodeJS.Timer | null = null;
    const RETRY_TIMEOUT = 30;

    $: if (timeout === RETRY_TIMEOUT) {
        if (interval) clearInterval(interval);

        interval = setInterval(() => timeout--, 1000);
    } else if (timeout === 0 && interval) {
        clearInterval(interval);
        interval = null;
    }

    async function test_domain() {
        loading = true;
        timeout = 30;

        let { status, body } = await fetch(`domain?id=${$onboarding.domain_id}`, {
            method: "GET"
        }).then(async res => ({
            status: res.status,
            body: await res.json().catch(() => ({}))
        }));

        if (status === 200) {
            go_to_account();
        } else {
            console.error(body);
        }

        loading = false;
    }

    function go_to_account() {
        goto("/dashboard");
    }
</script>

<Card name="Verify Domain">
    <p>To verify your ownership of the domain, add the following DNS records then click 'Verify'</p>

    <table>
        <thead>
            <tr>
                <td>Record Type</td>
                <td>Record Name</td>
                <td>Record Content</td>
            </tr>
        </thead>
        <tbody>
            {#each $onboarding.verification_codes as record}
                <tr>
                    <td>{record.record_type}</td>
                    <td>{record.name}</td>
                    <td>{record.value}</td>
                </tr>
            {/each}
        </tbody>
    </table>

    <button on:click={go_to_account}>Verify Later</button>
    <button default disabled={loading || timeout > 0} on:click={test_domain}>
        Verify
        {#if timeout > 0}
            (try again in {timeout} second{#if timeout === 1}s{/if})
        {/if}
    </button>
</Card>
