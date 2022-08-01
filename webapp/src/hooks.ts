import type { Handle } from "@sveltejs/kit";
import { verify } from "$lib/jwt";

export const handle: Handle = async ({ event, resolve }) => {
    let jwt = event.request.headers.get("cookie")
        ?.split(/\s*;\s*/)
        .find(cookie => cookie.startsWith("auth_token"))
        ?.split("=")
        ?.at(1) || null;

    let remove_token_cookie = false;

    if (jwt) {
        let user = await verify(jwt);

        if (user) event.locals.user = user;
        else remove_token_cookie = true;
    }

    let response = await resolve(event);

    if (remove_token_cookie) {
        response.headers.set("Set-Cookie", "auth_token=deleted; expires=Thu, 01 Jan 1970 00:00:00 GMT");
    }

    return response;
}
