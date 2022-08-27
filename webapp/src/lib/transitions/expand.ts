import type { TransitionConfig } from "svelte/transition";
import { cubicInOut } from "svelte/easing";

export default (node: HTMLElement, params: TransitionConfig) => {
    node.addEventListener("introstart", () => node.style.overflowY = "hidden");
    node.addEventListener("introend", () => node.style.overflowY = "");
    node.addEventListener("outrostart", () => node.style.overflowY = "hidden");
    node.addEventListener("outroend", () => node.style.overflowY = "");

    let height = node.scrollHeight;

    return {
        delay: params.delay || 0,
        duration: params.duration || 200,
        easing: params.easing || cubicInOut,
        css: (t: number, _u: number) => `max-height: ${height * t}px`
    }
}

export const action = (node: HTMLElement) => {
    // Make zero size
    node.style.overflowY = "hidden";
    node.style.height = "0px";

    requestAnimationFrame(() => {
        // Element has been created, animate in
        let original_height = node.scrollHeight;

        console.log("Starting");
        console.log(original_height);


        node.addEventListener("transitionend", () => {
            node.style.height = "";
            node.style.overflowY = "";
        }, true);

        // Prepare transition
        node.style.transition = "height 1s ease-in-out";

        // Trigger transition
        node.style.height = original_height + "px";
    });

    return {
        destroy: () => {
            // Element destroyed, animate out
            //node.style.overflowY = "hidden";
            //node.style.height = "0px";
        }
    }
}
