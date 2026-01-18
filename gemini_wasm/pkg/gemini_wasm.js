/**
 * ブラウザ上で進行するアプリ状態。
 */
export class App {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AppFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_app_free(ptr, 0);
    }
    /**
     * クリック入力（盤面座標）。合法手なら着手し true。
     * @param {number} x
     * @param {number} y
     * @returns {boolean}
     */
    click(x, y) {
        const ret = wasm.app_click(this.__wbg_ptr, x, y);
        return ret !== 0;
    }
    /**
     * 盤面上の黒石数を返す。
     * @returns {number}
     */
    count_black() {
        const ret = wasm.app_count_black(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * 盤面上の白石数を返す。
     * @returns {number}
     */
    count_white() {
        const ret = wasm.app_count_white(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * human（黒） vs `depth_white` の alphabeta（白）。
     */
    constructor() {
        const ret = wasm.app_new();
        this.__wbg_ptr = ret >>> 0;
        AppFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * パスを試みる（合法なら true）。
     * @returns {boolean}
     */
    pass() {
        const ret = wasm.app_pass(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * 盤面の描画イベントを発火する（実描画は JS 側）。
     */
    render() {
        wasm.app_render(this.__wbg_ptr);
    }
    /**
     * 黒を alphabeta に切り替える。
     * @param {number} depth
     */
    set_black_alphabeta(depth) {
        wasm.app_set_black_alphabeta(this.__wbg_ptr, depth);
    }
    /**
     * 黒を human に切り替える。
     */
    set_black_human() {
        wasm.app_set_black_human(this.__wbg_ptr);
    }
    /**
     * 黒を random に切り替える。
     * @param {bigint} seed
     */
    set_black_random(seed) {
        wasm.app_set_black_random(this.__wbg_ptr, seed);
    }
    /**
     * 白を alphabeta に切り替える。
     * @param {number} depth
     */
    set_white_alphabeta(depth) {
        wasm.app_set_white_alphabeta(this.__wbg_ptr, depth);
    }
    /**
     * 白を human に切り替える。
     */
    set_white_human() {
        wasm.app_set_white_human(this.__wbg_ptr);
    }
    /**
     * 白を random に切り替える。
     * @param {bigint} seed
     */
    set_white_random(seed) {
        wasm.app_set_white_random(this.__wbg_ptr, seed);
    }
    /**
     * 手番を返す。
     *
     * - 0=Black, 1=White, 255=Unknown
     * @returns {number}
     */
    side_to_move() {
        const ret = wasm.app_side_to_move(this.__wbg_ptr);
        return ret;
    }
    /**
     * ゲーム状態（勝敗）を返す。
     *
     * - 0=InProgress
     * - 1=Black wins
     * - 2=White wins
     * - 3=Draw
     * - 255=Unknown
     * @returns {number}
     */
    status_code() {
        const ret = wasm.app_status_code(this.__wbg_ptr);
        return ret;
    }
    /**
     * AI が動けるなら最大 `max_steps` 手だけ進める。実行した手数を返す。
     * @param {number} max_steps
     * @returns {number}
     */
    tick(max_steps) {
        const ret = wasm.app_tick(this.__wbg_ptr, max_steps);
        return ret >>> 0;
    }
    /**
     * AI 手番を 1 手だけ進める。
     *
     * - 実行した手数（0 or 1）を返す。
     * - 300ms 遅延などの「待ち」は JS 側の責務とする。
     * @returns {number}
     */
    tick_ai() {
        const ret = wasm.app_tick_ai(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) App.prototype[Symbol.dispose] = App.prototype.free;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_render_begin_4d1424e19c71e146: function() {
            window.render_begin();
        },
        __wbg_render_cell_0f2fbf31f1d3dfee: function(arg0, arg1, arg2) {
            window.render_cell(arg0, arg1, arg2);
        },
        __wbg_render_end_93757fcb73218c4d: function() {
            window.render_end();
        },
        __wbg_render_hint_ad4ff0898a77b8d8: function(arg0, arg1) {
            window.render_hint(arg0, arg1);
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./gemini_wasm_bg.js": import0,
    };
}

const AppFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_app_free(ptr >>> 0, 1));

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('gemini_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
