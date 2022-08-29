<script lang="ts">
    import DnsRecordTable from "../DnsRecordTable.svelte";
    import onboarding from "$lib/stores/onboarding";
</script>
<h1>Add CNAME Record</h1>

<p>
    In order to direct traffic from your domain to your new portfolio, you must
    add the following CNAME record to your domain. Without this, your portfolio
    won't work. Once this is added click 'Next'.
</p>

<DnsRecordTable
    disable_refresh
    records={[
        {
            record_type: "cname",
            name: $onboarding.domain,
            value: import.meta.env.VITE_CNAME_TARGET
        }
    ]}
/>

<button
    default
    on:click={() => $onboarding.step++}
>
    Next
</button>
