import { get_domain, link_domain } from "$lib/cloudflare";
import type { RequestHandler } from "@sveltejs/kit";

export const GET: RequestHandler = async ({ request, locals }) => {
    let { user } = locals;

    if (!user) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    let id = new URL(request.url).searchParams.get("id");

    if (!id) return {
        status: 400,
        headers: {},
        body: {
            message: "Malformed request"
        }
    }

    try {
        let domain_details = await get_domain(id);

        return {
            status: 200,
            headers: {},
            body: domain_details
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

export const POST: RequestHandler = async ({ request, locals }) => {
    let { user } = locals;

    if (!user) return {
        status: 403,
        headers: {},
        body: {
            message: "Unauthorized"
        }
    }

    let domain = (await request.json().catch(() => null))?.domain;

    if (!domain) return {
        status: 400,
        headers: {},
        body: {
            message: "Malformed request"
        }
    }

    try {
        let domain_details = await link_domain(domain, user);

        return {
            status: 200,
            headers: {},
            body: domain_details
        }
    } catch (e) {
        return {
            status: 500,
            headers: {},
            body: {
                message: e as string
            }
        }
    }
}
