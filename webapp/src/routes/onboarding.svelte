<script lang="ts">
    import Scroll from "$lib/components/Scroll.svelte";
    import onboarding from "$lib/stores/onboarding";

    import LinkDomain from "$lib/components/onboarding/LinkDomain.svelte";
    import Referral from "$lib/components/onboarding/Referral.svelte";
    import AddCname from "$lib/components/onboarding/AddCNAME.svelte";
    import VerifyDomain from "$lib/components/onboarding/VerifyDomain.svelte";
</script>

<div id="container">
    <Scroll
        items={[Referral, LinkDomain, AddCname, VerifyDomain]}
        bind:step={$onboarding.step}
        navigation={{
            click: false,
            swipe: false,
            keyboard: false,
            scroll: false
        }}
    />
</div>

<style>
    #container {
        height: 100%;
        margin: 0 var(--space-lg);
    }
</style>

