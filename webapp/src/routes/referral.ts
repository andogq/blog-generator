import type { RequestHandler } from "@sveltejs/kit";
import { redeem_referral_code } from "$lib/cloudflare";

export const POST: RequestHandler = async ({ request, locals }) => {
    // Extract code from request
    let code = await request.json()
        .then(body => body?.code)
        .catch(() => null);

    if (code) {
        // Attempt to get from KV store
        let success = await redeem_referral_code(code, locals.user).then(() => true).catch(() => false);

        if (success) {
            return {
                status: 200,
                body: {
                    message: "Referral code valid"
                }
            }
        } else {
            return {
                status: 400,
                body: {
                    message: "Referral code invalid or already redeemed"
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
