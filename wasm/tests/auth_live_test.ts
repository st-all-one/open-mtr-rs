import init, { OpenMtrWasm } from "../pkg/open_mtr_wasm.js";

// Obtém credenciais MTR-PE do .env
function loadEnv(): Record<string, string> {
    const out: Record<string, string> = {};
    try {
        const env = Deno.readTextFileSync("../.env");
        for (const l of env.split("\n")) {
            const idx = l.indexOf("=");
            if (idx > 0) out[l.slice(0, idx).trim()] = l.slice(idx + 1).trim().replace(/^"|"$/g, "");
        }
    } catch {
        // sem .env
    }
    return out;
}

function hasMtrPe(env: Record<string, string>): boolean {
    return Boolean(env["MTR_PE_CNPJ"]);
}

await init();

// ─── Auth clássica via WASM (Deno SSR) ────────────────────────────────────

Deno.test({ name: "auth_obter_token_mtr_pe", fn: async () => {
    const env = loadEnv();
    if (!hasMtrPe(env)) { console.log("  SKIP: MTR_PE_CNPJ not set"); return; }

    const client = new OpenMtrWasm();
    const params = JSON.stringify({
        tipo: "cpf_senha_unidade",
        cpf: env["MTR_PE_CPF_USER"],
        senha: env["MTR_PE_PASSWORD"],
        unidade: Number(env["MTR_PE_UNITY"]),
        cnpj: env["MTR_PE_CNPJ"],
    });

    const r = JSON.parse(await client.obter_token("mtr_pe", params));
    console.log("  obter_token ok:", r.ok);
    if (!r.ok) throw new Error(`obter_token falhou: ${r.mensagem ?? JSON.stringify(r)}`);
    if (!r.token || r.token.length < 20) throw new Error("token inválido");
    console.log(`  token: ${r.token.slice(0, 20)}... (${r.token.length} chars)`);

    // Usa o token para iniciar o cliente e consultar
    const initRes = JSON.parse(await client.init("mtr_pe", r.token, ""));
    console.log("  init ok:", initRes.ok);
    if (!initRes.ok) throw new Error(`init falhou: ${initRes.mensagem}`);
} });

// ─── AuthParams inválido → erro controlado ────────────────────────────────

Deno.test({ name: "auth_obter_token_params_invalidos", fn: async () => {
    const client = new OpenMtrWasm();
    try {
        await client.obter_token("mtr_pe", "not-json");
        throw new Error("esperava erro de JSON inválido");
    } catch (e) {
        console.log(`  params inválidos rejeitados: ${(e as Error).message.slice(0, 60)}`);
    }
} });
