<script lang="ts">
    import onboarding from "$lib/stores/onboarding";
    import Input from "$lib/components/Input.svelte";

    let loading = false;
    let code: string = "";
    let error: string;
    let notified = false;

    async function redeem_code() {
        error = "";
        loading = true;

        let response = await fetch("/referral", {
            method: "POST",
            body: JSON.stringify({
                code
            })
        });

        if (response.status === 200) {
            $onboarding.step++;
        } else {
            error = (await response.json().catch(() => null))?.message || "Problem making request";
        }

        loading = false;
    }

    async function set_referral_waiting() {
        loading = true;

        let response = await fetch("/user", {
            method: "PATCH",
            body: JSON.stringify({
                referral_waiting: true
            })
        });

        if (response.status === 200) {
            notified = true;
        }

        loading = false;
    }
</script>

<h1>Referral Code</h1>

<p>Enter your referral code to get started.</p>

<Input label="Referral Code" placeholder="very_secret_code" bind:value={code}/>

{#if error}
    <p>{error}</p>
{/if}

<button default on:click={redeem_code} disabled={loading || code.length == 0}>Next</button>

<div id="gap" />

<p>
    Don't have a referral code yet? Click the button below to be one of the
    first to know when more are available!
</p>

<button on:click={set_referral_waiting} disabled={loading || notified}>
    {#if notified}
        Notified!
    {:else}
        Notify Me
    {/if}
</button>

<style>
    #gap {
        padding: 1rem;
    }
</style>
