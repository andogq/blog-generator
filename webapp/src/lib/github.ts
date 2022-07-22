export function get_user(auth_token: string): Promise<string | null> {
    return fetch("https://api.github.com/user", {
        headers: {
            "Authorization": `token ${auth_token}`
        }
    })
        .then(res => res.json())
        .then(body => body.login)
        .catch(() => null);
}
