import type { RequestHandler } from "@sveltejs/kit";
import { get_user } from "$lib/github";
import { sign } from "$lib/jwt";
import prisma from "$lib/prisma";

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

            // Extract API token out of response
            let api_token = res?.access_token;

            if (api_token) {
                // Get user information from GitHub
                let user = await get_user(api_token);

                if (user) {
                    let username: string = user.login;

                    let destination: string;
                    let id: string;

                    // Check if the user exists in the database
                    if (await prisma.user.findUnique({
                        where: {
                            username
                        }
                    })) {
                        // Update user (login)
                        let user = await prisma.user.update({
                            where: {
                                username
                            },
                            data: {
                                api_token,
                                last_login: new Date()
                            }
                        });

                        id = user.id;
                        destination = "/dashboard";
                    } else {
                        // Create user (register)
                        let new_user = await prisma.user.create({
                            data: {
                                username: user.login,
                                api_token
                            }
                        });

                        id = new_user.id;
                        destination = "/onboarding";
                    }

                    // Create JWT
                    let jwt = await sign(id);

                    // Redirect to destination (onboarding or dashboard)
                    return {
                        status: 303,
                        headers: {
                            "Set-Cookie": `auth_token=${jwt}`,
                            "Location": destination
                        }
                    }
                } else {
                    return {
                        status: 500,
                        headers: {},
                        body: {
                            error: "Problem getting user data from GitHub"
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
