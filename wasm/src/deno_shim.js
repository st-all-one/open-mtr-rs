const memory = new WebAssembly.Memory({ initial: 256 });
export { memory };

export async function js_fetch(method, url, headersJson, bodyPtr, bodyLen) {
    const headers = new Headers();
    try {
        const parsed = JSON.parse(headersJson);
        for (const [k, v] of parsed) {
            headers.set(k, v);
        }
    } catch (_) {}

    const init = { method, headers };
    const memoryView = new Uint8Array(memory.buffer);

    if (bodyLen > 0) {
        init.body = new Uint8Array(memoryView.buffer.slice(bodyPtr, bodyPtr + bodyLen));
    }

    const resp = await fetch(url, init);
    const buf = new Uint8Array(await resp.arrayBuffer());

    return {
        status: resp.status,
        body: buf,
        contentType: resp.headers.get("content-type") || "",
        contentLength: resp.headers.get("content-length") || "",
    };
}
