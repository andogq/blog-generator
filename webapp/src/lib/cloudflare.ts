const WORKER_URL = process.env["WORKER_URL"] || "";
const WORKER_SECRET_TOKEN = process.env["WORKER_SECRET_TOKEN"] || "";

if (!WORKER_URL) throw new Error("Unable to find WORKER_URL in environment");
if (!WORKER_SECRET_TOKEN) throw new Error("Unable to find WORKER_SECRET_TOKEN in environment");

export enum Method {
    Get = "GET",
    Post = "POST",
    Delete = "DELETE"
};

export async function request(endpoint: string, { method, body }: {
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

export type DomainRecord = {
    record_type: string,
    name: string,
    value: string
};

export type DomainDetails = {
	id: string,
	hostname: string,
	dns_records: DomainRecord[],
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

export async function remove_domain(id: string): Promise<void> {
    let { status, body } = await request(`/cf/hostname/${id}`, { method: Method.Delete });

    if (status !== 200) {
        throw new Error(body?.message || "Problem deleting domain");
    }
}

