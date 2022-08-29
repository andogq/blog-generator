<script lang="ts">
    import { onMount, type ComponentType } from "svelte";

    const WHEEL_THRESHOLD = 100;
    const RESET_TIMEOUT = 500;

    export let items: ComponentType[];
    export let step = 0;

    $: if (step < 0) step = 0;
    $: if (step >= items.length && items.length !== 0) step = items.length - 1;

    export let navigation = {
        click: true,
        swipe: true,
        keyboard: true,
        scroll: true
    };

    let el_container: HTMLDivElement;
    let el_cards: HTMLElement[] = [];

    function on_key_event(e: KeyboardEvent) {
        if (e.key === "ArrowDown") {
            if (step + 1 < items.length) step++;
            e.preventDefault();
        } else if (e.key === "ArrowUp") {
            if (step > 0) step--;
            e.preventDefault();
        }
    }

    function on_mouse_click(i: number) {
        step = i;
    }

    let dy = 0;
    let reset_timeout: null | NodeJS.Timeout = null;
    function on_scroll_event(e: WheelEvent) {
        if (!(Math.sign(dy) === 1 && step === items.length - 1) && !(Math.sign(dy) === -1 && step == 0)) {
            dy += e.deltaY;
        }

        if (Math.abs(dy) > WHEEL_THRESHOLD) {
            step += Math.sign(dy);

            dy = 0;
        } else {
            if (reset_timeout !== null) clearTimeout(reset_timeout);

            reset_timeout = setTimeout(() => {
                dy = 0;
            }, RESET_TIMEOUT);
        }
    }
    
    $: if (el_cards[step]) {
        el_container.scrollTo({
            top: el_cards[step].offsetTop - (el_container.clientHeight / 2) + (el_cards[step].clientHeight / 2),
            left: 0,
            behavior: "smooth"
        });
    }

    // Allow arrow key navigation
    onMount(() => {
        el_container.focus();
    });
</script>

<div
    id="container"
    bind:this={el_container}
    on:keydown={navigation.keyboard ? on_key_event : undefined}
    on:wheel={navigation.scroll ? on_scroll_event : undefined}
    tabindex="0"
>
    <div class="spacer" />

    {#each items as Component, i}
        <div
            bind:this={el_cards[i]}
            class:active={step === i}
            on:click|self={navigation.click ? on_mouse_click.bind(undefined, i) : undefined}
            style="--translate: calc({dy / WHEEL_THRESHOLD} * 1rem)"
        >
            <Component />
        </div>
    {/each}

    <div class="spacer" />
</div>

<style>
    #container {
        height: 100%;
        overflow-y: hidden;

        padding: var(--space-sm);
    }
    #container:focus {
        outline: none;
    }

    .active {
        position: relative;
        top: var(--translate);
    }

    #container > *:not(.spacer) {
        transition: transform var(--transition), backdrop-filter var(--transition), top var(--transition);

        background: var(--background-light);
        border-radius: var(--border-radius);

        padding: var(--space-sm) var(--space-md);

        display: flex;
        flex-direction: column;
        gap: var(--space-xs);
    }

    #container > *:not(.active, .spacer) {
        opacity: 0.25;

        transform: scale(80%);
    }
    #container > *:not(.active, .spacer):after {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        content: "";
        backdrop-filter: blur(5px);
    }

    .spacer {
        height: 100%;
    }
</style>

