/**
 * Demonstração: consumo de 2 sistemas MTR simultaneamente via WASM no Deno.
 *
 * Padrão central: UMA INSTÂNCIA de OpenMtrWasm = UM SISTEMA.
 * Para unificar SINIR + MTR-PE no mesmo servidor, criamos uma instância por
 * sistema — cada uma com seu próprio token e base_url.
 *
 * Fluxo:
 *   MTR-PE (auth clássica):  obter_token("mtr_pe", params) → JWT → instância.init()
 *   SINIR (token integração): integração direta via .env      → instância.init()
 *
 * Uso:
 *   deno run --allow-read --allow-net examples/multi_system.ts
 */

import init, { OpenMtrWasm } from "../pkg/open_mtr_wasm.js";

// ─── Util ───────────────────────────────────────────────────────────────────

function loadEnv(): Record<string, string> {
    const out: Record<string, string> = {};
    try {
        const env = Deno.readTextFileSync("../.env");
        for (const l of env.split("\n")) {
            const idx = l.indexOf("=");
            if (idx > 0) out[l.slice(0, idx).trim()] = l.slice(idx + 1).trim().replace(/^"|"$/g, "");
        }
    } catch {
        // sem .env — fluxo demo fica parcial
    }
    return out;
}

/** Envelope Resposta padronizado pelo wrapper. */
interface Resposta {
    ok: boolean;
    mensagem: string;
    dados: any[];
    erros: { status?: number; detail?: string }[];
    total?: number | null;
}

function compactarMtr(m: any) {
    return {
        numero: m.numero,
        barcode: m.barcode ?? "",
        status: m.status,
        data_criacao: m.data_criacao,
        data_expedicao: m.data_expedicao ?? null,
        gerador: m.gerador?.cnpj ?? null,
        transportador: m.transportador?.cnpj ?? null,
        destinador: m.destinador?.cnpj ?? null,
        itens: m.residuos?.length ?? 0,
    };
}

// ─── Clientes por sistema ───────────────────────────────────────────────────

/** Cliente WASM do MTR-PE: obtém o token por auth clássica e já inicializa. */
async function criarClienteMtrPe(env: Record<string, string>): Promise<OpenMtrWasm> {
    const params = JSON.stringify({
        tipo: "cpf_senha_unidade",
        cpf: env["MTR_PE_CPF_USER"],
        senha: env["MTR_PE_PASSWORD"],
        unidade: Number(env["MTR_PE_UNITY"]),
        cnpj: env["MTR_PE_CNPJ"],
    });

    // 1. Geração do token via OpenMtrClassicAuth (dentro do WASM, sem reqwest)
    const auth = JSON.parse(await client_obter_token("mtr_pe", params));
    if (!auth.ok) throw new Error(`MTR-PE auth falhou: ${auth.mensagem ?? JSON.stringify(auth)}`);
    console.log(`   🔑 MTR-PE token: ${auth.token.slice(0, 20)}...`);

    // 2. Instância dedicada ao MTR-PE
    const cliente = new OpenMtrWasm();
    const initRes = JSON.parse(await cliente.init("mtr_pe", auth.token, ""));
    if (!initRes.ok) throw new Error(`MTR-PE init falhou: ${initRes.mensagem}`);
    return cliente;
}

/** Cliente WASM do SINIR: usa token de integração (sem auth clássica). */
async function criarClienteSinir(env: Record<string, string>): Promise<OpenMtrWasm> {
    const cliente = new OpenMtrWasm();
    const initRes = await cliente.init("sinir", env["SINIR_REFRESH_TOKEN"], "");
    const parsed = JSON.parse(initRes);
    if (!parsed.ok) throw new Error(`SINIR init falhou: ${parsed.mensagem}`);
    return cliente;
}

// helper p/ acessar obter_token fora de uma instância cliente específica
const client_obter_token = (s: string, p: string) => {
    const probe = new OpenMtrWasm();
    return probe.obter_token(s, p);
};

/** Consulta TODAS as páginas de um sistema, agregando em um único Resposta.
 *  Obs.: o SINIR limita cada página a 10 itens (ignora tamanho > 10). */
async function consultarTodasPaginado(
    cliente: OpenMtrWasm,
    filtroBase: Record<string, unknown>,
    maxPaginas = 100,
): Promise<Resposta> {
    const todos: any[] = [];
    const erros: any[] = [];
    let pagina = 1;
    let total: number | null = null;
    for (;;) {
        const filtro = JSON.stringify({ ...filtroBase, pagina, tamanho: 10 });
        const resp = JSON.parse(await cliente.consultar_mtrs(filtro)) as Resposta;
        if (!resp.ok) {
            return { ok: false, mensagem: resp.mensagem, dados: [], erros: [], total: 0 };
        }
        todos.push(...(resp.dados ?? []));
        erros.push(...(resp.erros ?? []));
        total = resp.total ?? todos.length;
        if (todos.length >= total || (resp.dados ?? []).length === 0 || pagina >= maxPaginas) break;
        pagina += 1;
    }
    return { ok: true, mensagem: "OK", dados: todos, erros, total };
}

// ─── Main ───────────────────────────────────────────────────────────────────

await init();
const env = loadEnv();

console.log("═".repeat(70));
console.log("  WASM + Deno — consumo simultâneo de 2 sistemas");
console.log("═".repeat(70));

const resultados: Record<string, Resposta> = {};

// Datas como objetos Date do JS — o Rust resolve internamente para o formato
// que cada sistema espera (dd-MM-yyyy no SINIR/MTR-PE).
// Obs.: o SINIR limita o período de pesquisa a 30 dias.
const fim = new Date();
const inicio = new Date();
inicio.setDate(inicio.getDate() - 30); // últimos 30 dias (limite SINIR)

const filtroBase = {
    periodo: { dataInicial: inicio, dataFinal: fim }, // aceita new Date() (ISO 8601)
    parte: { tipo: "Transportador" }, // a empresa é TRANSPORTADORA (tipoParceiro=5 no SINIR)
    detalhado: true, // traz resíduos/CNPJs — útil para consolidar
};

// ─── Sistema 1: MTR-PE ─────────────────────────────────────────────────────
if (env["MTR_PE_CNPJ"]) {
    console.log("\n🏭 MTR-PE (Pernambuco)");
    try {
        const mtrPe = await criarClienteMtrPe(env);
        const resp = await consultarTodasPaginado(mtrPe, filtroBase);
        resultados["mtr_pe"] = resp;
        if (!resp.ok) {
            console.log(`   ❌ ${resp.mensagem}`);
        } else {
            console.log(`   ✅ ${(resp.dados ?? []).length} MTRs (total: ${resp.total ?? "?"})`);
            (resp.dados ?? []).slice(0, 3).forEach((m) => console.log(`      · #${m.numero} ${m.status} ${compactarMtr(m).itens} itens`));
        }
    } catch (e) {
        console.log(`   ❌ ${(e as Error).message}`);
    }
} else {
    console.log("\n🏭 MTR-PE — SKIP (sem credenciais no .env)");
}

// ─── Sistema 2: SINIR ──────────────────────────────────────────────────────
if (env["SINIR_REFRESH_TOKEN"]) {
    console.log("\n🏭 SINIR (Nacional)");
    try {
        const sinir = await criarClienteSinir(env);
        // SINIR tem ~715 MTRs; exemplo limita a 2 páginas (20 itens) p/ não
        // sobrecarregar — use maxPaginas alto em produção.
        const resp = await consultarTodasPaginado(sinir, filtroBase, 2);
        resultados["sinir"] = resp;
        if (!resp.ok) {
            console.log(`   ❌ ${resp.mensagem}`);
        } else {
            console.log(`   ✅ ${(resp.dados ?? []).length} MTRs (total: ${resp.total ?? "?"})`);
            (resp.dados ?? []).slice(0, 3).forEach((m) => console.log(`      · #${m.numero} ${m.status} ${compactarMtr(m).itens} itens`));
        }
    } catch (e) {
        console.log(`   ❌ ${(e as Error).message}`);
    }
} else {
    console.log("\n🏭 SINIR — SKIP (sem token no .env)");
}

// ─── Consolidação ──────────────────────────────────────────────────────────
console.log("\n" + "═".repeat(70));
console.log("  📊 Painel consolidado");
console.log("═".repeat(70));
const porSistema: Record<string, { ok: boolean; mensagem: string; mtrs: number; erros: number }> = {};
for (const [sys, r] of Object.entries(resultados)) {
    porSistema[sys] = {
        ok: r.ok,
        mensagem: r.mensagem,
        mtrs: r.dados?.length ?? 0,
        erros: r.erros?.length ?? 0,
    };
}
const todos = Object.entries(resultados).flatMap(([sys, r]) =>
    (r.dados ?? []).map((m) => ({ sistema: sys, ...compactarMtr(m) }))
);
console.log(`  Total MTRs: ${todos.length}`);
const porStatus = todos.reduce<Record<string, number>>((acc, m) => {
    acc[m.status] = (acc[m.status] ?? 0) + 1;
    return acc;
}, {});
console.log(`  Por status: ${JSON.stringify(porStatus)}`);
for (const sys of Object.keys(porSistema)) {
    const s = porSistema[sys];
    console.log(`  ${sys}: ${s.ok ? `${s.mtrs} MTRs, ${s.erros} erros` : `❌ ${s.mensagem}`}`);
}

console.log("\n  ✅ Done.");
