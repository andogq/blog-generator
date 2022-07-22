<script lang="ts">
    import Card from "./Card.svelte";
    import onboarding from "$lib/stores/onboarding";
</script>

<Card name="Verify Domain">
    <p>To verify your ownership of the domain, add the following DNS records then click 'Verify'</p>

    <table>
        <thead>
            <tr>
                <td>Record Type</td>
                <td>Record Name</td>
                <td>Record Content</td>
            </tr>
        </thead>
        <tbody>
            {#each $onboarding.verification_codes as record}
                <tr>
                    <td>{record.type}</td>
                    <td>{record.name}</td>
                    <td>{record.content}</td>
                </tr>
            {/each}
        </tbody>
    </table>

    <button disabled>Try Again Later</button>
    <button default>Verify</button>
</Card>
