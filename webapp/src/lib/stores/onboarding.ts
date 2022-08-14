import { writable, type Writable } from "svelte/store";

const store: Writable<{
    step: number,
    domain: string,
    domain_id: string,
    verification_codes: {
        record_type: string,
        name: string,
        value: string
    }[]
}> = writable({
    step: 0,
    domain: "",
    domain_id: "",
    verification_codes: []
});

export default store;
