export function prettify_text(text: string) {
    return text.split("_").map(w => w[0].toUpperCase() + w.slice(1)).join(" ");
}
