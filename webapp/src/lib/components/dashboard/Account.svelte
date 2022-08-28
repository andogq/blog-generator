<script>
    import ButtonList from "$lib/components/ButtonList.svelte";
</script>

<h1>Account</h1>

<ButtonList
    options={[
        {
            name: "Log Out",
            description: "Log out within this browser. This won't affect any deployed sites.",
            button_text: "Log Out",
            on_click: () => {}
        },
        {
            name: "Delete Everything",
            description: "This will delete everything, including revoking GitHub authorization and domain links, causing any deployed sites to stop working.",
            button_text: "Delete Everything",
            on_click: () => {}
        }
    ]}
/>

