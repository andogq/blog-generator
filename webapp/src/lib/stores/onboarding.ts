import { writable, type Writable } from "svelte/store";

const store: Writable<{
    step: number,
    domain: string,
    verification_codes: {
        type: string,
        name: string,
        content: string
    }[]
}> = writable({
    step: 0,
    domain: "",
    verification_codes: []
});

export default store;
