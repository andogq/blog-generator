import prisma from "$lib/prisma";

const WORKER_URL = process.env["WORKER_URL"] || "";
const WORKER_SECRET_TOKEN = process.env["WORKER_SECRET_TOKEN"] || "";

if (!WORKER_URL) throw new Error("Unable to find WORKER_URL in environment");
if (!WORKER_SECRET_TOKEN) throw new Error("Unable to find WORKER_SECRET_TOKEN in environment");

enum Method {
    Get = "GET",
    Post = "POST"
};

async function request(endpoint: string, { method, body }: {
    method?: Method,
    body?: any
} = { method: Method.Get, body: undefined }) {
    try {
        const res = await fetch(`${WORKER_URL}/_${endpoint}`, {
            headers: {
                "Authorization": `Bearer ${WORKER_SECRET_TOKEN}`
            },
            method,
            body
        });

        return ({
            body: await res.json(),
            status: res.status
        });
    } catch (e) {
        console.error("Problem making request to worker:");
        console.error(e);

        throw new Error("Error with request");
    }
}


export async function add_auth_token(username: string, token: string): Promise<boolean> {
    let { status, body } = await request(`/kv/auth_tokens/${username}`, {
        method: Method.Post,
        body: token
    });

    return status === 200;
}

export async function redeem_referral_code(code: string, user: string): Promise<void> {
    let { status, body } = await request(`/do/referral_code/${code}/use`, {
        method: Method.Post,
        body: user
    });

    if (status === 200) {
        return;
    } else {
        throw new Error(body?.message || "Invalid response");
    }
}

type DomainDetails = {
	id: string,
	hostname: string,
	dns_records: {
        record_type: string,
        name: string,
        value: string
    }[],
	verification_status: string,
	ssl_status: string,
	errors: string[]
}

export async function get_domain(id: string): Promise<DomainDetails> {
    let { status, body: domain_details } = await request(`/cf/hostname/${id}`, { method: Method.Get });

    if (status === 200) {
        return domain_details;
    } else {
        throw new Error(domain_details?.message || "Problem getting domain");
    }
}

export async function link_domain(domain: string, user: string): Promise<DomainDetails> {
    let { status, body: domain_details } = await request(`/cf/hostname/${domain}`, { method: Method.Post });

    if (status === 200) {
        let { status, body } = await request(`/kv/domains/${domain}`, {
            method: Method.Post,
            body: user
        });

        if (status === 200) {
            return domain_details;
        } else {
            throw new Error(body?.message || "Problem setting domain");
        }
    } else { 
        throw new Error(domain_details?.message || "Invalid response");
    }
}

