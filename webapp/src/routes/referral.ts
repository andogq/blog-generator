import type { RequestHandler } from "@sveltejs/kit";
import prisma from "$lib/prisma";

export const POST: RequestHandler = async ({ request, locals }) => {
    // Extract code from request
    let code = await request.json()
        .then(body => body?.code)
        .catch(() => null);

    if (code) {
        let success = await prisma.$transaction(async prisma => {
            // Check if referral code is valid
            let referral_code = await prisma.referralCode.findUnique({
                where: {
                    code
                },
                select: {
                    count: true,
                    _count: {
                        select: {
                            users: true
                        }
                    }
                }
            });

            if (referral_code && referral_code.count > referral_code._count.users) {
                // Use code if it is
                await prisma.user.update({
                    where: {
                        username: locals.user
                    },
                    data: {
                        s_referral_code: code
                    }
                });

                return true;
            } else return false;
        });

        return {
            status: success ? 200 : 400,
            headers: {},
            body: {
                success
            }
        }
    } else return {
        status: 400,
        body: {
            message: "Invalid code"
        }
    }
}
