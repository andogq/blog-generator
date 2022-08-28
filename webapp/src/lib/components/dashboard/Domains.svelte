<script lang="ts">
    import { browser } from "$app/env";
    import Input from "$lib/components/Input.svelte";
    import PopOver from "$lib/components/PopOver.svelte";
    import { prettify_text } from "$lib/helpers";
    import { expand } from "$lib/transitions";

    import DnsRecordTable from "$lib/components/DnsRecordTable.svelte";

    type DnsRecord = {
        record_type: string,
        name: string,
        value: string
    }

    type Domain = {
        id: string
        hostname: string
        dns_records: DnsRecord[]
        verification_status: string
        ssl_status: string
        errors: string[]
    }

    let domains: Domain[] = [];
    let managed_domain: Domain | null = null;

    $: deletion_confirm_open = false && managed_domain;
    let feedback: string;
    let domain_deletion_confirmation: string;

    let loading = false;

    if (browser) {
        refresh_domains();
    }

    async function refresh_domains() {
        loading = true;

        domains = await fetch("/user")
            .then(res => res.json())
            .then(user => Promise.all(
                user.domains.map((domain: string) => (
                    fetch(`/domain/${domain}`).then(res => res.json())
                ))
            ));

        loading = false;
    }

    function open_deletion_confirmation() {
        if (!deletion_confirm_open) {
            feedback = "";
            domain_deletion_confirmation = "";
        }

        deletion_confirm_open = !deletion_confirm_open;
    }

    async function delete_domain() {
        if (managed_domain) {
            loading = true;

            await fetch(`/domain/${managed_domain.id}`, {
                method: "DELETE",
                body: JSON.stringify({
                    feedback
                })
            });

            await refresh_domains();
            managed_domain = null;

            loading = false;
        }
    }
</script>

<h1>Domains</h1>

<table>
    <thead>
        <tr>
            <td>Domain</td>
            <td>Verification Status</td>
            <td>SSL Status</td>
            <td>Actions</td>
        </tr>
    </thead>
    <tbody>
    {#each domains as domain}
        <tr>
            <td><a href="https://{domain.hostname}">{domain.hostname}</a></td>
            <td>
                {prettify_text(domain.verification_status)}
            </td>
            <td>
                {prettify_text(domain.ssl_status)}
            </td>
            <td>
                <button
                    on:click={() => managed_domain = domain}
                >
                    Manage
                </button>
            </td>
        </tr>
    {/each}
    </tbody>
</table>

<PopOver
    title={managed_domain?.hostname || ""}
    open={managed_domain !== null}
    width="60vw"
    on:close={() => managed_domain = null}
>
    {#if managed_domain}
        <h3>Summary</h3>

        <div>
            <p>Verification Status: <b>{prettify_text(managed_domain.verification_status)}</b></p>
            <p>SSL Status: <b>{prettify_text(managed_domain.ssl_status)}</b></p>
        </div>

        <h3>Verification</h3>
        <DnsRecordTable
            records={managed_domain.dns_records}
            {loading}
            on:refresh={refresh_domains}
        />

        <h3>Danger Zone</h3>

        {#if deletion_confirm_open}
            <div id="domain_delete_confirm" transition:expand>
                <p>
                    Warning! Deleting the domain will immediately remove the domain, potentially resulting
                    in a broken website! To prevent this, please make sure that you have done the following
                    before you remove the domain:
                </p>

                <div>
                    <ol>
                        <li>Set up a website with another provider</li>
                        <li>Log into your DNS provider, and remove the CNAME record pointing to this service</li>
                        <li>
                            Insert the required DNS records for your new provider (should be an A, AAAA, CNAME
                            record or something similar)
                        </li>
                        <li>Wait a few minutes for changes to propagate, and test if you are able to access your new website</li>
                    </ol>
                </div>

                <p>
                    That should be it! If you can access your new site you will be safe to delete the domain below.
                    We're sorry to see you go, we would love to hear from you if you have any feedback, suggestions,
                    or have encountered a problem that has made you want to jump ship (this is completely optional).
                </p>

                <Input label="What's making you leave?" placeholder="I really had a problem with....." bind:value={feedback} />

                <p>To acknowledge that you know what you're about to do, please type out your domain name below:</p>
                <Input label="Domain Name" placeholder={managed_domain.hostname} bind:value={domain_deletion_confirmation} />

                <button on:click={delete_domain} disabled={domain_deletion_confirmation !== managed_domain.hostname || loading}>Delete Domain</button>
            </div>
        {:else}
            <button on:click={open_deletion_confirmation} on:click={delete_domain}>Delete Domain</button>
        {/if}
    {/if}
</PopOver>

<style>
    table {
        border-spacing: 0.5rem;
    }

    thead {
        font-weight: bold;
    }

    #domain_delete_confirm {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
</style>
