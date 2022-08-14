<script lang="ts">
    import LinkDomain from "$lib/components/onboarding/LinkDomain.svelte";
    import Referral from "$lib/components/onboarding/Referral.svelte";
    import VerifyDomain from "$lib/components/onboarding/VerifyDomain.svelte";
    import { onMount } from "svelte";
    import onboarding from "$lib/stores/onboarding";

    let cards: HTMLElement[] = [];
    let card_container: HTMLDivElement;

    onMount(() => {
        document.body.addEventListener("keydown", e => {
            if (e.key === "ArrowDown") {
                if ($onboarding.step + 1 < cards.length) $onboarding.step++;
                e.preventDefault();
            } else if (e.key === "ArrowUp") {
                if ($onboarding.step > 0) $onboarding.step--;
                e.preventDefault();
            }
        });
    });

    $: if (cards[$onboarding.step]) {
        card_container.scrollTo({
            top: cards[$onboarding.step].offsetTop - (card_container.clientHeight / 2) + (cards[$onboarding.step].clientHeight / 2),
            left: 0,
            behavior: "smooth"
        });
    }
</script>

<div id="scroll" bind:this={card_container}>
    <div class="spacer" />
    {#each [Referral, LinkDomain, VerifyDomain] as Card, i}
        <div bind:this={cards[i]} class:active={$onboarding.step === i}>
            <Card />
        </div>
    {/each}
    <div class="spacer" />
</div>

<style>
    #scroll {
        height: 100%;
        overflow-y: hidden;

        padding: var(--space-sm);
    }

    #scroll > * {
        transition: transform var(--transition), backdrop-filtern var(--transition);
    }

    #scroll > *:not(.active) {
        opacity: 0.25;
        position: relative;

        transform: scale(80%);
    }
    #scroll > *:not(.active, .spacer):after {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        content: "";
        top: -20px;
        left: -20px;
        right: -20px;
        bottom: -20px;
        backdrop-filter: blur(5px);
    }

    .spacer {
        height: 100%;
    }
</style>
