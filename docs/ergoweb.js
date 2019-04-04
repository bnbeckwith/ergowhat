/* tslint:disable */
import * as wasm from './ergoweb_bg';

const slab = [{ obj: undefined }, { obj: null }, { obj: true }, { obj: false }];

let slab_next = slab.length;

function addHeapObject(obj) {
    if (slab_next === slab.length) slab.push(slab.length + 1);
    const idx = slab_next;
    const next = slab[idx];
    
    slab_next = next;
    
    slab[idx] = { obj, cnt: 1 };
    return idx << 1;
}

export function __wbg_static_accessor_document_document() {
    return addHeapObject(document);
}

const __wbg_getElementById_8c4314de7fabbd92_target = HTMLDocument.prototype.getElementById  || function() {
    throw new Error(`wasm-bindgen: HTMLDocument.prototype.getElementById does not exist`);
} ;

const stack = [];

function getObject(idx) {
    if ((idx & 1) === 1) {
        return stack[idx >> 1];
    } else {
        const val = slab[idx >> 1];
        
        return val.obj;
        
    }
}

const TextDecoder = typeof self === 'object' && self.TextDecoder
    ? self.TextDecoder
    : require('util').TextDecoder;

let cachedDecoder = new TextDecoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

function getStringFromWasm(ptr, len) {
    return cachedDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

export function __wbg_getElementById_8c4314de7fabbd92(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    return addHeapObject(__wbg_getElementById_8c4314de7fabbd92_target.call(getObject(arg0), varg1));
}

function GetOwnOrInheritedPropertyDescriptor(obj, id) {
    while (obj) {
        let desc = Object.getOwnPropertyDescriptor(obj, id);
        if (desc) return desc;
        obj = Object.getPrototypeOf(obj);
    }
    throw new Error(`descriptor for id='${id}' not found`);
}

const __wbg_set_inner_html_fc05625f561eb4f2_target = GetOwnOrInheritedPropertyDescriptor(Element.prototype, 'innerHTML').set  || function() {
    throw new Error(`wasm-bindgen: GetOwnOrInheritedPropertyDescriptor(Element.prototype, 'innerHTML').set does not exist`);
} ;

export function __wbg_set_inner_html_fc05625f561eb4f2(arg0, arg1, arg2) {
    let varg1 = getStringFromWasm(arg1, arg2);
    __wbg_set_inner_html_fc05625f561eb4f2_target.call(getObject(arg0), varg1);
}

const TextEncoder = typeof self === 'object' && self.TextEncoder
    ? self.TextEncoder
    : require('util').TextEncoder;

let cachedEncoder = new TextEncoder('utf-8');

function passStringToWasm(arg) {
    
    const buf = cachedEncoder.encode(arg);
    const ptr = wasm.__wbindgen_malloc(buf.length);
    getUint8Memory().set(buf, ptr);
    return [ptr, buf.length];
}
/**
* @param {string} arg0
* @param {string} arg1
* @returns {void}
*/
export function make_svg(arg0, arg1) {
    const [ptr0, len0] = passStringToWasm(arg0);
    const [ptr1, len1] = passStringToWasm(arg1);
    try {
        return wasm.make_svg(ptr0, len0, ptr1, len1);
        
    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);
        wasm.__wbindgen_free(ptr1, len1 * 1);
        
    }
    
}

function dropRef(idx) {
    
    idx = idx >> 1;
    if (idx < 4) return;
    let obj = slab[idx];
    
    obj.cnt -= 1;
    if (obj.cnt > 0) return;
    
    // If we hit 0 then free up our space in the slab
    slab[idx] = slab_next;
    slab_next = idx;
}

export function __wbindgen_object_drop_ref(i) {
    dropRef(i);
}

export function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

