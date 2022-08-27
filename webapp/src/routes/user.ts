import prisma from "$lib/prisma";
import { Status } from "@prisma/client";
import type { RequestHandler } from "@sveltejs/kit";

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
                id: user.id,
                username: user.username,
                date_created: user.date_created,
                last_login: user.last_login,
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
