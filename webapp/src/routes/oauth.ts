import type { RequestHandler } from "@sveltejs/kit";
import { add_auth_token } from "$lib/cloudflare";
import { get_user } from "$lib/github";
import { sign, verify } from "$lib/jwt";

export const GET: RequestHandler = async ({ url }) => {
    let code = url.searchParams.get("code");
    let client_secret = process.env["GITHUB_CLIENT_SECRET"];
    let client_id = process.env["VITE_GITHUB_CLIENT_ID"];

    if (client_secret && client_id) {
        if (code) {
            // Make second request of oauth
            let url = new URL("https://github.com/login/oauth/access_token");
            url.search = new URLSearchParams({
                client_id,
                client_secret,
                code
            }).toString();

            let res = await fetch(url.href, {
                method: "POST",
                headers: {
                    "Accept": "application/json"
                }
            }).then(res => res.json()).catch(null);

            let access_token = res?.access_token;

            if (access_token) {
                let user = await get_user(access_token);

                if (user) {
                    let response = await add_auth_token(user, access_token);

                    if (response.success) {
                        // Create JWT
                        let jwt = await sign(user);

                        return {
                            status: 303,
                            headers: {
                                "Set-Cookie": `auth_token=${jwt}`,
                                "Location": "/onboarding"
                            }
                        }
                    } else {
                        console.error("Problem with cloudflare request");
                        console.error(response);

                        return {
                            status: 500,
                            headers: {},
                            body: {
                        
                            }
                        }
                    }
                } else {
                    return {
                        status: 500,
                        headers: {},
                        body: {
                    
                        }
                    }
                }
            } else {
                console.error("Unknown response from github");
                console.error(res);

                return {
                    status: 500,
                    headers: {},
                    body: {
                        error: "Problem with response from GitHub"
                    }
                }
            }
        } else return {
            status: 200,
            headers: {},
            body: {
                error: "Problem performing OAuth with GitHub"
            }
        }
    } else return {
        status: 500,
        headers: {},
        body: {
            error: "Internal server error"
        }
    }
}
