/** @type {import('./__types/index.d.ts').RequestHandler} */
export async function GET(request) {
    let magic_number = Math.random();

    return {
        status: 200,
        headers: {},
        body: { magic_number }
    }
}
