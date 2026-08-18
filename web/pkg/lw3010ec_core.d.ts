/* tslint:disable */
/* eslint-disable */

export function build_frame(unit_id: number, func_code: number, address: number, quantity_or_value: number): Uint8Array;

export function crc16(data: Uint8Array): number;

export function modbus_read_holding(): number;

export function modbus_write_single(): number;

export function parse_response(data: Uint8Array, expected_unit_id: number, expected_func_code: number, expected_quantity: number): any;

export function reg_current_read(): number;

export function reg_current_write(): number;

export function reg_output_read(): number;

export function reg_output_write(): number;

export function reg_voltage_read(): number;

export function reg_voltage_write(): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly build_frame: (a: number, b: number, c: number, d: number) => [number, number];
    readonly crc16: (a: number, b: number) => number;
    readonly modbus_read_holding: () => number;
    readonly modbus_write_single: () => number;
    readonly parse_response: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly reg_current_read: () => number;
    readonly reg_current_write: () => number;
    readonly reg_output_read: () => number;
    readonly reg_output_write: () => number;
    readonly reg_voltage_read: () => number;
    readonly reg_voltage_write: () => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
