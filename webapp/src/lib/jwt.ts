import * as jose from "jose";

const JWT_SECRET = process.env["JWT_SECRET"] || "";
if (!JWT_SECRET) throw new Error("JWT_SECRET not found in environment");

const SECRET = Uint8Array.from(Buffer.from(JWT_SECRET, "base64"));

export function sign(id: string): Promise<string> {
    return new jose.SignJWT({ id })
        .setProtectedHeader({ alg: 'HS256' })
        .sign(SECRET);
}

export async function verify(jwt: string): Promise<string> {
    return jose.jwtVerify(jwt, SECRET)
        .then(({ payload }) => {
            if (typeof payload?.id === "string") return payload.id;
            else throw new Error("Invalid user ID");
        })
        .catch(() => {
            throw new Error("Invalid user ID");
        });
}
