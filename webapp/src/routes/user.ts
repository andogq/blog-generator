import prisma from "$lib/prisma";
import { Status, type User } from "@prisma/client";
import type { RequestHandler } from "@sveltejs/kit";

function stripped_user(user: User) {
    return {
        id: user.id,
        username: user.username,
        date_created: user.date_created,
        last_login: user.last_login,
    }
}

export const GET: RequestHandler = async ({ locals }) => {
    let user = locals.user;

    if (user) {
        let domains = await prisma.domain.findMany({
            where: {
                user,
                status: Status.ACTIVE
            },
            select: {
                cloudflare_id: true,
            }
        }).then(domains => domains.map(d => d.cloudflare_id));

        let referral_codes = await prisma.referralCode.findMany({
            where: {
                creator: user
            },
            select: {
                code: true
            }
        }).then(codes => codes.map(c => c.code));

        return {
            status: 200,
            headers: {},
            body: {
                ...stripped_user(user),
                domains,
                referral_codes
            }
        }
    } else {
        return {
            status: 400,
            headers: {},
            body: {}
        }
    }
}

export const PATCH: RequestHandler = async ({ request, locals }) => {
    let user = locals.user;

    if (user === null) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    let body = await request.json().catch(() => null);

    if (body === null) return {
        status: 400,
        headers: {},
        body: {
            message: "Bad request"
        }
    }

    let user_updates: {
        referral_waiting?: boolean
    } = {};

    if (body.referral_waiting === true || body.referral_waiting === false) {
        user_updates.referral_waiting = body.referral_waiting;
    }

    let new_user = await prisma.user.update({
        where: {
            username: user.username
        },
        data: user_updates as any // Really bad way to do this
    });

    return {
        status: 200,
        headers: {},
        body: stripped_user(new_user),
    }
}

