/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const abiVersion: () => number;
export const callJson: (a: number, b: number, c: number, d: number) => [number, number, number, number];
export const callVec: (
  a: number,
  b: number,
  c: number,
  d: number,
  e: number,
  f: number,
  g: number,
  h: number,
) => [number, number, number, number];
export const initPanicHook: () => void;
export const manifest: () => [number, number];
export const run: (
  a: number,
  b: number,
  c: number,
  d: number,
  e: number,
  f: number,
) => [number, number, number, number];
export const version: () => [number, number];
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __externref_table_alloc: () => number;
export const __wbindgen_start: () => void;
