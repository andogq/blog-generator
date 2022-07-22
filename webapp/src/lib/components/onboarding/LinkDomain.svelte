<script lang="ts">
    import Card from "./Card.svelte";
    import Input from "$lib/components/Input.svelte";

    let domain: string;
</script>

<Card name="Link Domain">
    <p>Enter the domain you would like to use below.</p>

    <Input label="Domain" placeholder="www.example.com" value={domain} />
    
    <button default>Next</button>
</Card>
