/* tslint:disable */
/* eslint-disable */

export class OpenMtrWasm {
    free(): void;
    [Symbol.dispose](): void;
    aceitar_alteracoes(json: string): Promise<string>;
    cancelar_mtrs(json: string): Promise<string>;
    consultar_cdfs(json: string): Promise<string>;
    consultar_mtrs(filtro_json: string): Promise<string>;
    criar_manifesto_complementar(json: string): Promise<string>;
    criar_mtrs(json: string): Promise<string>;
    download_cdfs(json: string): Promise<string>;
    download_mtrs(json: string): Promise<string>;
    gerar_cdfs(json: string): Promise<string>;
    health(): Promise<string>;
    init(system: string, token: string, base_url: string): Promise<string>;
    listar_acondicionamentos(): Promise<string>;
    listar_acondicionamentos_por_estado_fisico(codigo_estado: number): Promise<string>;
    listar_classes(): Promise<string>;
    listar_classes_por_residuo(codigo_ibama: string): Promise<string>;
    listar_estados_fisicos(): Promise<string>;
    listar_residuos(): Promise<string>;
    listar_tratamentos(): Promise<string>;
    listar_unidades(): Promise<string>;
    modificar_mtrs(json: string): Promise<string>;
    constructor();
    receber_mtrs(json: string): Promise<string>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_openmtrwasm_free: (a: number, b: number) => void;
    readonly openmtrwasm_aceitar_alteracoes: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_cancelar_mtrs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_consultar_cdfs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_consultar_mtrs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_criar_manifesto_complementar: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_criar_mtrs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_download_cdfs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_download_mtrs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_gerar_cdfs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_health: (a: number) => number;
    readonly openmtrwasm_init: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly openmtrwasm_listar_acondicionamentos: (a: number) => number;
    readonly openmtrwasm_listar_acondicionamentos_por_estado_fisico: (a: number, b: number) => number;
    readonly openmtrwasm_listar_classes: (a: number) => number;
    readonly openmtrwasm_listar_classes_por_residuo: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_listar_estados_fisicos: (a: number) => number;
    readonly openmtrwasm_listar_residuos: (a: number) => number;
    readonly openmtrwasm_listar_tratamentos: (a: number) => number;
    readonly openmtrwasm_listar_unidades: (a: number) => number;
    readonly openmtrwasm_modificar_mtrs: (a: number, b: number, c: number) => number;
    readonly openmtrwasm_new: () => number;
    readonly openmtrwasm_receber_mtrs: (a: number, b: number, c: number) => number;
    readonly __wasm_bindgen_func_elem_608: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_905: (a: number, b: number, c: number, d: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
