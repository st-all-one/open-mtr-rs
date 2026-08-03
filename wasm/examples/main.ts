import init, { OpenMtrWasm } from "../pkg/open_mtr_wasm.js";

let token = "";
try {
    const env = Deno.readTextFileSync("../.env");
    for (const l of env.split("\n")) {
        if (l.startsWith("OPENMTR_SINIR_TOKEN=")) {
            token = l.split("=").slice(1).join("=").trim();
            break;
        }
    }
} catch {
    console.error("No .env file found");
}

if (!token) {
    console.error("OPENMTR_SINIR_TOKEN not found in .env");
    Deno.exit(1);
}

console.log("Initializing WASM module...");
await init();

console.log("Creating client...");
const client = new OpenMtrWasm();

console.log("Calling init...");
const initResult = JSON.parse(await client.init("sinir", token, ""));
console.log("init:", initResult);

if (!initResult.ok) {
    console.error("Init failed:", initResult.mensagem);
    Deno.exit(1);
}

console.log("\n--- lista de resíduos ---");
const residuos = JSON.parse(await client.listar_residuos());
console.log("OK:", residuos.ok, "Total:", residuos.dados?.length, "Erros:", residuos.erros?.length);
if (residuos.dados) {
    for (const r of residuos.dados.slice(0, 5)) {
        console.log(`  ${r.codigo_ibama}: ${r.descricao ?? "—"}`);
    }
}

console.log("\n--- consultar MTRs (últimos 7 dias) ---");
const filtro = JSON.stringify({
    periodo: { dias: 7 },
    tamanho: 5,
    detalhado: false,
});
const mtrs = JSON.parse(await client.consultar_mtrs(filtro));
console.log("OK:", mtrs.ok, "Total:", mtrs.dados?.length, "Erros:", mtrs.erros?.length);
if (mtrs.dados) {
    for (const m of mtrs.dados.slice(0, 3)) {
        console.log(`  MTR #${m.numero} status=${m.status} data=${m.data_criacao}`);
    }
}

console.log("\n--- listar unidades ---");
const unidades = JSON.parse(await client.listar_unidades());
console.log("OK:", unidades.ok, "Total:", unidades.dados?.length);

console.log("\n--- listar estados_fisicos ---");
const efs = JSON.parse(await client.listar_estados_fisicos());
console.log("OK:", efs.ok, "Total:", efs.dados?.length);

console.log("\n--- health ---");
const h = JSON.parse(await client.health());
console.log(h);

console.log("\nDone.");
