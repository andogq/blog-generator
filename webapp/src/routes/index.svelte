<script lang="ts">
    import { page } from "$app/stores";

    const GITHUB_OAUTH_URL = new URL("https://github.com/login/oauth/authorize");
    GITHUB_OAUTH_URL.search = new URLSearchParams({
        client_id: import.meta.env.VITE_GITHUB_CLIENT_ID,
        redirect_uri: new URL("/oauth", $page.url.origin).href
    }).toString();
</script>

<div id="content">
    <h1>Welcome!</h1>

    <div id="text">
        <p>To get started, <a href={GITHUB_OAUTH_URL.href}>login with GitHub</a>.</p>
    </div>
</div>

<style>
    #content {
        margin: var(--space-md) var(--space-lg);

        display: flex;
        flex-direction: column;
        gap: var(--space-sm);
    }

    #text {
        background: var(--background-light);
        border-radius: var(--border-radius);
        padding: var(--space-sm);
    }
</style>

