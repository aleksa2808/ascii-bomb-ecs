/* tslint:disable */
/* eslint-disable */

export function run(): void;

export function set_input_active(input: number): void;

export function start_game(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly set_input_active: (a: number) => void;
    readonly start_game: () => void;
    readonly run: () => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___closure__destroy___dyn_core_10728d4c20eeb4de___ops__function__FnMut__core_10728d4c20eeb4de___option__Option_web_sys_3934d01d52738957___features__gen_Blob__Blob_____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___closure__destroy___dyn_core_10728d4c20eeb4de___ops__function__FnMut__wasm_bindgen_59a928b5ac64dd9e___JsValue____Output_______: (a: number, b: number) => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___convert__closures_____invoke___js_sys_ced2c258707ecdde___Array__web_sys_3934d01d52738957___features__gen_ResizeObserver__ResizeObserver_____: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___convert__closures_____invoke___js_sys_ced2c258707ecdde___Array_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___convert__closures_____invoke___wasm_bindgen_59a928b5ac64dd9e___JsValue_____: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen_59a928b5ac64dd9e___convert__closures_____invoke______: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
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
