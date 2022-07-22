<script lang="ts">
    let domains = [
        {
            domain: "ando.gq",
            active: true
        },
        {
            domain: "ando.sh",
            active: false
        }
    ];
</script>

<h1>Domains</h1>

<table>
    <thead>
        <tr>
            <td>Domain</td>
            <td>Status</td>
        </tr>
    </thead>
    <tbody>
    {#each domains as domain}
        <tr>
            <td><a href="https://{domain.domain}">{domain.domain}</a></td>
            <td>
                {#if domain.active}
                    Active
                {:else}
                    Pending Verification
                    <!-- svelte-ignore a11y-missing-attribute -->
                    <a>(retry)</a>
                {/if}
            </td>
        </tr>
    {/each}
    </tbody>
</table>

<style>
    thead {
        font-weight: bold;
    }
</style>
