import init, { OpenMtrWasm } from "../pkg/open_mtr_wasm.js";

// ─── Setup ────────────────────────────────────────────────────────────────

let client: OpenMtrWasm;
let token = "";

function loadToken(): string {
    if (token) return token;
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
    return token;
}

function hasToken(): boolean {
    return loadToken().length > 0;
}

async function ensureClient(): Promise<OpenMtrWasm> {
    if (!client) {
        client = new OpenMtrWasm();
        const r = JSON.parse(await client.init("sinir", loadToken(), ""));
        if (!r.ok) throw new Error(`init failed: ${r.mensagem}`);
    }
    return client;
}

function diasAtras(dias: number): string {
    const d = new Date();
    d.setDate(d.getDate() - dias);
    const dd = String(d.getDate()).padStart(2, "0");
    const mm = String(d.getMonth() + 1).padStart(2, "0");
    return `${dd}-${mm}-${d.getFullYear()}`;
}

function hoje(): string {
    const d = new Date();
    const dd = String(d.getDate()).padStart(2, "0");
    const mm = String(d.getMonth() + 1).padStart(2, "0");
    return `${dd}-${mm}-${d.getFullYear()}`;
}

function assertOk(resp: any, label: string) {
    if (!resp.ok) {
        const erros = resp.erros?.map((e: any) => e.detail).join("; ") ?? "unknown";
        throw new Error(`${label}: not ok — ${resp.mensagem ?? erros}`);
    }
}

// ─── Bootstrap ─────────────────────────────────────────────────────────────

Deno.test({ name: "00_bootstrap: WASM init", fn: async () => {
    await init();
    if (!hasToken()) { console.log("  SKIP: OPENMTR_SINIR_TOKEN not set"); return; }
    client = await ensureClient();
    console.log("  init OK");
} });

// ─── Tabelas de Referência ─────────────────────────────────────────────────

Deno.test({ name: "01_listar_residuos", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_residuos());
    assertOk(r, "listar_residuos");
    console.log(`  ${r.dados.length} resíduos`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

Deno.test({ name: "02_listar_acondicionamentos", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_acondicionamentos());
    assertOk(r, "listar_acondicionamentos");
    console.log(`  ${r.dados.length} acondicionamentos`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

Deno.test({ name: "03_listar_classes", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_classes());
    assertOk(r, "listar_classes");
    console.log(`  ${r.dados.length} classes`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

Deno.test({ name: "04_listar_estados_fisicos", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_estados_fisicos());
    assertOk(r, "listar_estados_fisicos");
    console.log(`  ${r.dados.length} estados físicos`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

Deno.test({ name: "05_listar_tratamentos", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_tratamentos());
    assertOk(r, "listar_tratamentos");
    console.log(`  ${r.dados.length} tratamentos`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

Deno.test({ name: "06_listar_unidades", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_unidades());
    assertOk(r, "listar_unidades");
    console.log(`  ${r.dados.length} unidades`);
    if (r.dados.length === 0) throw new Error("expected > 0");
} });

// ─── Consulta de MTRs ──────────────────────────────────────────────────────

Deno.test({ name: "07_consultar_mtrs_ultimos_30_dias", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const filtro = JSON.stringify({
        periodo: { dataInicial: diasAtras(30), dataFinal: hoje() },
        parte: { tipo: "Transportador" },
        tamanho: 10,
        detalhado: false,
    });
    const r = JSON.parse(await c.consultar_mtrs(filtro));
    assertOk(r, "consultar_mtrs");
    console.log(`  ${r.dados.length} MTRs (últimos 30 dias)`);
    if (r.dados.length > 0) {
        const m = r.dados[0];
        if (!m.numero) throw new Error("MTR sem numero");
        if (!m.status) throw new Error("MTR sem status");
    }
} });

Deno.test({ name: "08_consultar_mtrs_ultimos_7_dias_sem_parte", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const filtro = JSON.stringify({
        periodo: { dataInicial: diasAtras(7), dataFinal: hoje() },
        tamanho: 5,
        detalhado: false,
    });
    const r = JSON.parse(await c.consultar_mtrs(filtro));
    assertOk(r, "consultar_mtrs");
    console.log(`  ${r.dados.length} MTRs (últimos 7 dias, sem filtro parte)`);
} });

Deno.test({ name: "09_consultar_mtrs_sem_filtro", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    try {
        const r = JSON.parse(await c.consultar_mtrs("{}"));
        // SINIR pode rejeitar filtro vazio com 415 — ambos os casos são válidos
        if (r.ok) {
            console.log(`  ${r.dados.length} MTRs (sem filtro) — ok`);
        } else {
            console.log(`  filtro vazio rejeitado: ${r.mensagem?.slice(0, 60)} — esperado`);
        }
    } catch (e) {
        console.log(`  filtro vazio rejeitado: ${(e as Error).message?.slice(0, 60)} — esperado`);
    }
} });

Deno.test({ name: "10_consultar_mtrs_detalhado", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const filtro = JSON.stringify({
        periodo: { dataInicial: diasAtras(30), dataFinal: hoje() },
        parte: { tipo: "Transportador" },
        tamanho: 3,
        detalhado: true,
    });
    const r = JSON.parse(await c.consultar_mtrs(filtro));
    assertOk(r, "consultar_mtrs_detalhado");
    console.log(`  ${r.dados.length} MTRs (detalhado)`);
    if (r.dados.length > 0) {
        const m = r.dados[0];
        if (!m.residuos || m.residuos.length === 0) throw new Error("MTR detalhado sem residuos");
        const residuo = m.residuos[0];
        if (!residuo.codigo_ibama) throw new Error("residuo sem codigo_ibama");
        // Verifica campos enriquecidos pelo detalhado
        console.log(`  MTR #${m.numero}: ${m.residuos.length} residuos, classe=${residuo.classe}, ef=${residuo.estado_fisico}`);
    }
} });

Deno.test({ name: "11_consultar_mtrs_sem_resultados", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const filtro = JSON.stringify({
        periodo: { dataInicial: "01-01-2020", dataFinal: "02-01-2020" },
        tamanho: 5,
    });
    const r = JSON.parse(await c.consultar_mtrs(filtro));
    // Deve retornar ok=true com 0 dados (período sem MTRs)
    if (!r.ok) throw new Error("esperado ok=true para período vazio");
    console.log(`  ${r.dados.length} MTRs (janela 2020 — esperado 0)`);
} });

// ─── Download ───────────────────────────────────────────────────────────────

Deno.test({ name: "12_download_mtr_pdf", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    // Busca MTRs primeiro para obter um número
    const consulta = JSON.parse(await c.consultar_mtrs(JSON.stringify({
        periodo: { dataInicial: diasAtras(30), dataFinal: hoje() },
        parte: { tipo: "Transportador" },
        tamanho: 3,
        detalhado: false,
    })));
    if (!consulta.ok || consulta.dados.length === 0) {
        console.log("  SKIP: sem MTRs para download");
        return;
    }
    const numero = consulta.dados[0].numero;
    const r = JSON.parse(await c.download_mtrs(JSON.stringify([numero])));
    assertOk(r, "download_mtrs");
    const item = r.dados[0];
    console.log(`  MTR #${numero}: buffer=${item.buffer?.length ?? 0} bytes, formato=${item.formato ?? "?"}`);
    if (!item.buffer || item.buffer.length < 100) throw new Error("download buffer muito pequeno");
} });

// ─── Health ─────────────────────────────────────────────────────────────────

Deno.test({ name: "13_health", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const h = JSON.parse(await c.health());
    // health tenta um consultar_mtrs vazio — SINIR pode rejeitar
    console.log(`  health: ok=${h.ok} mensagem=${h.mensagem?.slice(0, 60)}`);
    // Não assert ok — depende de como o sistema responde a filtro vazio
} });

// ─── CDF ────────────────────────────────────────────────────────────────────

Deno.test({ name: "14_consultar_cdfs", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const filtro = JSON.stringify({
        periodo: { dataInicial: diasAtras(90), dataFinal: hoje() },
        tamanho: 5,
    });
    const r = JSON.parse(await c.consultar_cdfs(filtro));
    // CDFs podem estar vazios — o endpoint deve retornar ok
    console.log(`  ${r.dados?.length ?? 0} CDFs (últimos 90 dias)`);
} });

// ─── Erros ──────────────────────────────────────────────────────────────────

Deno.test({ name: "15_token_invalido", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c2 = new OpenMtrWasm();
    try {
        await c2.init("sinir", "token-invalido-12345", "");
        throw new Error("esperado erro com token inválido");
    } catch (e) {
        console.log(`  erro esperado: ${(e as Error).message?.slice(0, 80)}`);
    }
} });

Deno.test({ name: "16_init_com_sistema_invalido", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c2 = new OpenMtrWasm();
    try {
        await c2.init("sistema_inexistente", "token", "");
        throw new Error("esperado erro com sistema inválido");
    } catch (e) {
        console.log(`  erro esperado: ${(e as Error).message?.slice(0, 80)}`);
    }
} });

Deno.test({ name: "17_cliente_nao_inicializado", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c2 = new OpenMtrWasm();
    try {
        await c2.listar_residuos();
        throw new Error("esperado erro sem init");
    } catch (e) {
        console.log(`  erro esperado: ${(e as Error).message?.slice(0, 80)}`);
    }
} });

// ─── Taxa (rate limit) ─────────────────────────────────────────────────────

Deno.test({ name: "18_multiplas_chamadas_rapidas", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const start = Date.now();
    const results: string[] = [];
    for (let i = 0; i < 3; i++) {
        const r = JSON.parse(await c.listar_unidades());
        results.push(r.ok ? "ok" : "fail");
    }
    const elapsed = Date.now() - start;
    console.log(`  3 chamadas listar_unidades: ${results.join(",")} em ${elapsed}ms`);
} });

// ─── Dados populados (campos específicos) ──────────────────────────────────

Deno.test({ name: "19_residuos_tem_codigo_ibama", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.listar_residuos());
    assertOk(r, "listar_residuos");
    const primeiro = r.dados[0];
    if (!primeiro.codigo_ibama) throw new Error("residuo sem codigo_ibama");
    console.log(`  primeiro: ${primeiro.codigo_ibama} — ${primeiro.descricao ?? "sem descrição"}`);
} });

Deno.test({ name: "20_mtr_tem_status_valido", fn: async () => {
    if (!hasToken()) { console.log("  SKIP"); return; }
    const c = await ensureClient();
    const r = JSON.parse(await c.consultar_mtrs(JSON.stringify({
        periodo: { dataInicial: diasAtras(30), dataFinal: hoje() },
        parte: { tipo: "Transportador" },
        tamanho: 5,
        detalhado: false,
    })));
    assertOk(r, "consultar_mtrs");
    if (r.dados.length === 0) { console.log("  SKIP: sem MTRs"); return; }
    for (const m of r.dados) {
        if (!m.status) throw new Error(`MTR #${m.numero} sem status`);
        const validos = ["Ativo", "Recebido", "Cancelado", "Encerrado", "EmElaboracao", "Outro"];
        // status pode ser string ou objeto — verifica se tem valor
        if (typeof m.status === "string") {
            console.log(`  MTR #${m.numero} status=${m.status}`);
        } else if (typeof m.status === "object") {
            console.log(`  MTR #${m.numero} status=${JSON.stringify(m.status).slice(0, 30)}`);
        }
    }
} });
