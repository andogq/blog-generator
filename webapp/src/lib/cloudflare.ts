import Cloudflare from "cloudflare";

const CLOUDFLARE_TOKEN = process.env["CLOUDFLARE_TOKEN"] || "";
const CLOUDFLARE_ACCOUNT_ID = process.env["CLOUDFLARE_ACCOUNT_ID"] || "";
const CLOUDFLARE_KV_DOMAIN_NAMESPACE_ID = process.env["CLOUDFLARE_KV_DOMAIN_NAMESPACE_ID"] || "";
const CLOUDFLARE_KV_AUTH_TOKENS_NAMESPACE_ID = process.env["CLOUDFLARE_KV_AUTH_TOKENS_NAMESPACE_ID"] || "";

if (!CLOUDFLARE_TOKEN) throw new Error("Unable to find CLOUDFLARE_TOKEN in environment");
if (!CLOUDFLARE_ACCOUNT_ID) throw new Error("Unable to find CLOUDFLARE_ACCOUNT_ID in environment");
if (!CLOUDFLARE_KV_DOMAIN_NAMESPACE_ID) throw new Error("Unable to find CLOUDFLARE_KV_DOMAIN_NAMESPACE_ID in environment");
if (!CLOUDFLARE_KV_AUTH_TOKENS_NAMESPACE_ID) throw new Error("Unable to find CLOUDFLARE_KV_AUTH_TOKENS_NAMESPACE_ID in environment");

const cloudflare = new Cloudflare({
    token: CLOUDFLARE_TOKEN
});


export function add_auth_token(user: string, token: string): Promise<{ success: boolean, error: string[], messages: string[] }> {
    return cloudflare.enterpriseZoneWorkersKV.add(CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_KV_AUTH_TOKENS_NAMESPACE_ID, user, token) as any;
}

