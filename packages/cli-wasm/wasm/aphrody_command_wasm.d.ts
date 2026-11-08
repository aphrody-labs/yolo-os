/* tslint:disable */
/* eslint-disable */

/**
 * Version of the shared RAG call surface.
 */
export function abiVersion(): number;

/**
 * Run a structured RAG operation; input and output are JSON documents.
 *
 * # Errors
 * Throws the operation's error message.
 */
export function callJson(op: string, input: string): string;

/**
 * Run a dense-vector operation over little-endian byte buffers.
 *
 * # Errors
 * Throws the operation's error message.
 */
export function callVec(op: string, params: string, a: Uint8Array, b: Uint8Array): Uint8Array;

/**
 * Install standard browser console error panic hook for debuggability and readable traces.
 */
export function initPanicHook(): void;

/**
 * JSON manifest: ABI, crate, version and every RAG operation name.
 */
export function manifest(): string;

/**
 * Run `aphrody pure <args...>`; `files_json` is a JSON object `{ "name": "text", ... }`.
 *
 * # Errors
 * Throws when `files_json` is not a JSON object of strings.
 */
export function run(args: string[], stdin: string, files_json: string): string;

/**
 * Package version of the pure command crate.
 */
export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly abiVersion: () => number;
  readonly callJson: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly callVec: (
    a: number,
    b: number,
    c: number,
    d: number,
    e: number,
    f: number,
    g: number,
    h: number,
  ) => [number, number, number, number];
  readonly initPanicHook: () => void;
  readonly manifest: () => [number, number];
  readonly run: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number, number];
  readonly version: () => [number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __externref_table_alloc: () => number;
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
export default function __wbg_init(
  module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>,
): Promise<InitOutput>;
