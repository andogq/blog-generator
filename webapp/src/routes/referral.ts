import type { RequestHandler } from "@sveltejs/kit";
import prisma from "$lib/prisma";
import { Status } from "@prisma/client";

export const POST: RequestHandler = async ({ request, locals }) => {
    let user = locals.user;

    if (!user) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    // Extract code from request
    let code = await request.json()
        .then(body => body?.code)
        .catch(() => null);

    if (code) {
        try {
            await prisma.$transaction(async prisma => {
                // Check if referral code is valid
                let referral_code = await prisma.referralCode.findUnique({
                    where: {
                        code
                    },
                    select: {
                        count: true,
                        status: true,
                        _count: {
                            select: {
                                users: true
                            }
                        }
                    }
                });

                if (referral_code && referral_code.status == Status.ACTIVE && referral_code.count > referral_code._count.users) {
                    // Use code if it is
                    await prisma.user.update({
                        where: {
                            id: locals.user.id
                        },
                        data: {
                            s_referral_code: code
                        }
                    });
                } else throw new Error("Referral code as expired or has been redeemed");
            });

            return {
                status: 200,
                headers: {},
                body: {
                    message: "Referral code successfully redeemed"
                }
            }
        } catch (error: unknown) {
            let message = "Error while redeeming referral code";

            if (error instanceof Error) message = error.message;
            else if (typeof error === "string") message = error;

            return {
                status: 400,
                headers: {},
                body: {
                    message
                }
            }
        }
    } else return {
        status: 400,
        body: {
            message: "Invalid code"
        }
    }
}
