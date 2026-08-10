import init, { OpenMtrWasm } from "../pkg/open_mtr_wasm.js";

let token = "";
let mtrPe: Record<string, string> = {};
try {
    const env = Deno.readTextFileSync("../.env");
    for (const l of env.split("\n")) {
        const idx = l.indexOf("=");
        if (idx <= 0) continue;
        const key = l.slice(0, idx).trim();
        const val = l.slice(idx + 1).trim().replace(/^"|"$/g, "");
        if (key === "OPENMTR_SINIR_TOKEN") token = val;
        if (key.startsWith("MTR_PE_")) mtrPe[key] = val;
    }
} catch {
    console.error("No .env file found");
}

console.log("Initializing WASM module...");
await init();

console.log("Creating client...");
const client = new OpenMtrWasm();

// ─── MTR-PE: autenticação clássica (CPF + senha) → token ───────────────────
if (mtrPe["MTR_PE_CNPJ"]) {
    console.log("\n--- obter_token (MTR-PE, autenticação clássica) ---");
    const params = JSON.stringify({
        tipo: "cpf_senha_unidade",
        cpf: mtrPe["MTR_PE_CPF_USER"],
        senha: mtrPe["MTR_PE_PASSWORD"],
        unidade: Number(mtrPe["MTR_PE_UNITY"]),
        cnpj: mtrPe["MTR_PE_CNPJ"],
    });
    const auth = JSON.parse(await client.obter_token("mtr_pe", params));
    console.log("OK:", auth.ok, "token:", auth.token ? `${auth.token.slice(0, 20)}...` : "—");
    if (auth.ok) {
        const initRes = JSON.parse(await client.init("mtr_pe", auth.token, ""));
        console.log("init MTR-PE:", initRes.ok, initRes.mensagem);
    }
}

// ─── SINIR: token de integração ─────────────────────────────────────────────
if (!token) {
    console.log("\n⚠️  OPENMTR_SINIR_TOKEN não encontrado no .env — pulando SINIR");
} else {
    console.log("\n--- init SINIR ---");
    const initResult = JSON.parse(await client.init("sinir", token, ""));
    console.log("init:", initResult);
}

console.log("\nDone.");
