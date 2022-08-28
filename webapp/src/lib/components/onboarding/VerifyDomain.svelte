<script lang="ts">
    import Card from "./Card.svelte";
    import onboarding from "$lib/stores/onboarding";
    import { goto } from "$app/navigation";
    import { prettify_text } from "$lib/helpers";
    import DnsRecordTable from "$lib/components/DnsRecordTable.svelte";

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

        let { body } = await fetch(`domain/${$onboarding.domain_id}`, {
            method: "GET"
        }).then(async res => ({
            status: res.status,
            body: await res.json().catch(() => ({}))
        }));

        $onboarding.ssl_status = body?.ssl_status || $onboarding.ssl_status;
        $onboarding.verification_status = body?.verification_status || $onboarding.verification_status;

        // Check to see if the verification status is appropriate
        if (body?.verification_status === "active" && body?.ssl_status === "active") {
            go_to_account();
        } else if (body?.dns_records) {
            // Refresh DNS records incase they've changed
            $onboarding.verification_codes = body.dns_records;
        }

        loading = false;
    }

    function go_to_account() {
        goto("/dashboard");
    }
</script>

<Card name="Verify Domain">
    <p>To verify your ownership of the domain, add the following DNS records then click 'Verify'</p>

    {#if $onboarding.verification_status}
        <p>Verification Status: <b>{prettify_text($onboarding.verification_status)}</b></p>
    {/if}
    {#if $onboarding.ssl_status}
        <p>SSL Status: <b>{prettify_text($onboarding.ssl_status)}</b></p>
    {/if}

    <DnsRecordTable
        records={$onboarding.verification_codes}
        {loading}
        disable_refresh
    />

    <button on:click={go_to_account}>Verify Later</button>
    <button default disabled={loading || timeout > 0} on:click={test_domain}>
        Check Verification
        {#if timeout > 0}
            (try again in {timeout} second{#if timeout !== 1}s{/if})
        {/if}
    </button>
</Card>
