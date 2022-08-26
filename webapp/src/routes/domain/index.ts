import { Method } from "$lib/cloudflare";
import type { RequestHandler } from "@sveltejs/kit";
import { request as cloudflare_request, type DomainDetails } from "$lib/cloudflare";
import prisma from "$lib/prisma";

export const POST: RequestHandler = async ({ request, locals }) => {
    let { user } = locals;

    // Make sure user is authenticated
    if (!user) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    // Extract the domain from the request
    let domain = (await request.json().catch(() => null))?.domain;

    if (!domain || typeof domain !== "string") return {
        status: 400,
        headers: {},
        body: {
            message: "Malformed request"
        }
    }

    // TODO: Make sure user has valid referral code

    try {
        // Make request with Cloudflare (via worker)
        let { status, body: domain_details } = await cloudflare_request(`/cf/hostname/${domain}`, { method: Method.Post }) as { status: number, body: DomainDetails };

        if (status === 200) {
            // Success! Add to database
            await prisma.domain.create({
                data: {
                    s_user: user.id,
                    domain,
                    cloudflare_id: domain_details.id,
                    hostname_status: domain_details.verification_status,
                    ssl_status: domain_details.ssl_status
                }
            });

            return {
                status: 200,
                headers: {},
                body: domain_details
            }
        } else {
            // Something went wrong
            return {
                status: 500,
                headers: {},
                body: {
                    message: "Problem adding domain"
                }
            }
        }

    } catch (e) {
        console.error(e);

        return {
            status: 500,
            headers: {},
            body: {
                message: "Internal error"
            }
        }
    }
}
