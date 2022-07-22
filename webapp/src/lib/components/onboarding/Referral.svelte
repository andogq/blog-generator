<script lang="ts">
    import Card from "$lib/components/onboarding/Card.svelte";
    import Input from "$lib/components/Input.svelte";

    let code: string;
</script>

<Card name="Referral Code">
    <p>Enter your referral code to get started.</p>

    <Input label="Referral Code" placeholder="very_secret_code" value={code} />

    <button default>Next</button>
</Card>

