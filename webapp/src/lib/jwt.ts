import jose from "jose";

const JWT_SECRET = process.env["JWT_SECRET"] || "";
if (!JWT_SECRET) throw new Error("JWT_SECRET not found in environment");

const SECRET = Uint8Array.from(Buffer.from(JWT_SECRET, "base64"));

export function sign(user: string): Promise<string> {
    return new jose.SignJWT({ user })
        .setProtectedHeader({ alg: 'HS256' })
        .sign(SECRET);
}

export function verify(jwt: string): Promise<string | null> {
    return jose.jwtVerify(jwt, SECRET)
        .then(({ payload }) => payload?.user as string || null)
        .catch(() => null);
}
