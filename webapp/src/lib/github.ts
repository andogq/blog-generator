export function get_user(auth_token: string): Promise<any | null> {
    return fetch("https://api.github.com/user", {
        headers: {
            "Authorization": `token ${auth_token}`
        }
    })
        .then(res => res.json())
        .catch(() => null);
}
