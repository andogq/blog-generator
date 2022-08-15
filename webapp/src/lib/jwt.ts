import jose from "jose";

const JWT_SECRET = process.env["JWT_SECRET"] || "";
if (!JWT_SECRET) throw new Error("JWT_SECRET not found in environment");

const SECRET = Uint8Array.from(Buffer.from(JWT_SECRET, "base64"));

export function sign({ username, id }: { username: string, id: string }): Promise<string> {
    return new jose.SignJWT({ username, id })
        .setProtectedHeader({ alg: 'HS256' })
        .sign(SECRET);
}

export function verify(jwt: string): Promise<{ username: string, id: string } | null> {
    return jose.jwtVerify(jwt, SECRET)
        .then(({ payload }) => {
            if (typeof payload?.username === "string" && typeof payload?.id === "string") {
                return { username: payload.username, id: payload.id };
            } else return null;
        })
        .catch(() => null);
}
