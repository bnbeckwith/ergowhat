extern crate ergodox_keymap_parser;

use std::{mem,str};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void};

use ergodox_keymap_parser::*;

fn main() {
}

// Allocate a buffer of the right size and forget about it so that is
// isn't automatically deallocated
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

// We are transferring our ownership of the underlying pointer to the
// vector below. This allows it to be deallocated automatically.
#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

// We are transferring our ownership of the underlying pointer to the
// string below. This allows it to be deallocated automatically.
#[no_mangle]
pub extern "C" fn dealloc_str(ptr: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

// The JavaScript side passes a pointer to a C-like string that's already placed into memory.
// On the Rust side we turn this into a CStr, extract the bytes, pass it through the crate
// and then turn it back into an memory-allocated C-like string.
// A pointer to this data is returned.
#[no_mangle]
pub extern "C" fn svg(cptr: *mut c_char) -> *mut c_char {
    unsafe {
        let raw = CStr::from_ptr(cptr);
        let contents = str::from_utf8(raw.to_bytes()).unwrap();

        let svg = to_svg(contents);
        
        let s = CString::new(svg).unwrap();
        s.into_raw()
    }
}
