<script>
    import ButtonList from "$lib/components/ButtonList.svelte";
</script>

<h1>Cache</h1>

<ButtonList
    options={[
        {
            name: "Clear Cache",
            description: "Something not showing up correctly? Try clear the cache to ensure that your latest profile information is being retrieved.",
            button_text: "Clear Cache",
            on_click: () => {}
        }
    ]}
/>

