import type { Handle } from "@sveltejs/kit";
import { verify } from "$lib/jwt";
import prisma from "$lib/prisma";

export const handle: Handle = async ({ event, resolve }) => {
    let jwt = event.request.headers.get("cookie")
        ?.split(/\s*;\s*/)
        .find(cookie => cookie.startsWith("auth_token"))
        ?.split("=")
        ?.at(1) || null;

    let remove_token_cookie = true;

    if (jwt) {
        try {
            let user_id = await verify(jwt);
            
            let user = await prisma.user.findUniqueOrThrow({
                where: {
                    id: user_id
                }
            });

            event.locals.user = user;
            remove_token_cookie = false;
        } catch (_) {}
    }

    let response = await resolve(event);

    if (remove_token_cookie) {
        response.headers.set("Set-Cookie", "auth_token=deleted; expires=Thu, 01 Jan 1970 00:00:00 GMT");
    }

    return response;
}
