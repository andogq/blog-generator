import { get_domain, type DomainRecord, Method, request as cf_request } from "$lib/cloudflare";
import prisma from "$lib/prisma";
import type { RequestHandler } from "./__types/[id]";

export const GET: RequestHandler = async ({ request, params, locals }) => {
    let { user } = locals;

    // Make sure user is authenticated
    if (!user) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    let id = params.id;

    if (!id) return {
        status: 400,
        headers: {},
        body: {
            message: "Malformed request"
        }
    }

    // Attempt to get domain details from the database
    let domain = await prisma.domain.findUnique({
        where: {
            cloudflare_id: id
        }
    });

    if (domain && domain.s_user === user.id) {
        // Found domain
        let force_update = new URL(request.url).searchParams.get("refresh") !== null;

        let dns_records: DomainRecord[] = [];
        let errors: string[] = [];

        if (force_update || domain.ssl_status !== "active" || domain.hostname_status !== "active") {
            // Get latest details from Cloudflare
            try {
                let cf_domain = await get_domain(id);
                dns_records = cf_domain.dns_records;
                errors = cf_domain.errors;

                // Save new status to database
                domain = await prisma.domain.update({
                    where: { cloudflare_id: id },
                    data: {
                        hostname_status: cf_domain.verification_status,
                        ssl_status: cf_domain.ssl_status,
                        date_updated: new Date()
                    }
                });

                // If activated, update KV store
                if (domain.ssl_status === "active" && domain.hostname_status === "active") {
                    let { status } = await cf_request(`/kv/domains/${domain.domain}`, {
                        method: Method.Post,
                        body: user.username
                    });
                    
                    if (status !== 200) throw new Error(`Status adding domain ${domain.domain} to store: ${status}`);
                }
            } catch (e) {
                console.log(e);
                return {
                    status: 500,
                    headers: {},
                    body: {
                        message: e as string
                    }
                }
            }
        }

        return {
            status: 200,
            headers: {},
            body: {
                id: domain.cloudflare_id,
                hostname: domain.domain,
                dns_records,
                verification_status: domain.hostname_status,
                ssl_status: domain.ssl_status,
                errors
            }
        }
    } else {
        // Domain doesn't exist
        return {
            status: 404,
            headers: {},
            body: {
                message: "Domain does not exist"
            }
        }
    }
}
