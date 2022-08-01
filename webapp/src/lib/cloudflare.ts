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


export async function add_auth_token(user: string, token: string): Promise<boolean> {
    let { status, body } = await request(`/kv/auth_tokens/${user}`, {
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

