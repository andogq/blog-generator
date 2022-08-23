<script lang="ts">
    import Card from "./Card.svelte";
    import Input from "$lib/components/Input.svelte";
    import onboarding from "$lib/stores/onboarding";

    let loading = false;
    let error: string;

    const DOMAIN_REGEX = /^.+\.\w+$/;
    $: valid_domain = DOMAIN_REGEX.test($onboarding.domain);

    async function link_domain() {
        loading = true;

        let { status, body } = await fetch("domain", {
            method: "POST",
            body: JSON.stringify({
                domain: $onboarding.domain
            })
        }).then(async res => ({
            status: res.status,
            body: await res.json().catch(() => ({}))
        }));

        if (status === 200) {
            $onboarding.verification_codes = body.dns_records;
            $onboarding.domain_id = body.id;
            $onboarding.ssl_status = body.ssl_status;
            $onboarding.verification_status = body.verification_status;
            $onboarding.step++;
        } else {
            error = body;
        }

        loading = false;
    }
</script>

<Card name="Link Domain">
    <p>Enter the domain you would like to use below.</p>

    <Input label="Domain" placeholder="www.example.com" bind:value={$onboarding.domain} />

    {#if error}
        <p>{error}</p>
    {/if}
    
    <button default on:click={link_domain} disabled={loading || !valid_domain}>Next</button>
</Card>
