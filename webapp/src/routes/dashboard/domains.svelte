<script lang="ts">
    import { browser } from "$app/env";

    let domains: any[] = [];

    if (browser) {
        fetch("/user")
            .then(res => res.json())
            .then(user => Promise.all(
                user.domains.map((domain: string) => (
                    fetch(`/domain/${domain}`).then(res => res.json())
                ))
            ))
            .then(_domains => domains = _domains);
    }
</script>

<h1>Domains</h1>

<table>
    <thead>
        <tr>
            <td>Domain</td>
            <td>Verification Status</td>
            <td>SSL Status</td>
        </tr>
    </thead>
    <tbody>
    {#each domains as domain}
        <tr>
            <td><a href="https://{domain.hostname}">{domain.hostname}</a></td>
            <td>
                {domain.verification_status}
            </td>
            <td>
                {domain.ssl_status}
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
