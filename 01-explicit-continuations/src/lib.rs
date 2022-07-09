#![allow(non_snake_case)]

use core::alloc::Layout;
use core::ffi::c_void;
use libc;
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut u8, x: i32);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> i64;

    #[link_name = "roc__mainForHost_1_Main_caller"]
    fn call_Main(flags: *const i32, closure_data: *const u8, output: *mut u8);

    #[allow(dead_code)]
    #[link_name = "roc__mainForHost_1_Main_size"]
    fn size_Main() -> i64;

    #[link_name = "roc__mainForHost_1_Main_result_size"]
    fn size_Main_result() -> i64;

    #[link_name = "roc__mainForHost_1_Continuation_caller"]
    fn call_Continuation(flags: *const i32, closure_data: *const u8, output: *mut i32);
}

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    libc::malloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    libc::realloc(c_ptr, new_size)
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    libc::free(c_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    match tag_id {
        0 => {
            let slice = CStr::from_ptr(c_ptr as *const c_char);
            let string = slice.to_str().unwrap();
            eprintln!("Roc hit a panic: {}", string);
            std::process::exit(1);
        }
        _ => todo!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_memcpy(dst: *mut c_void, src: *mut c_void, n: usize) -> *mut c_void {
    libc::memcpy(dst, src, n)
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    libc::memset(dst, c, n)
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    let size = unsafe { roc_main_size() } as usize;
    let layout = Layout::array::<u8>(size).unwrap();

    let x = 21;
    let y = 15;
    let z = -9;
    unsafe {
        // TODO allocate on the stack if it's under a certain size
        let buffer = std::alloc::alloc(layout);

        roc_main(buffer, x);

        let (cont_buffer, cont_layout) = call_main_closure(buffer, y);
        std::alloc::dealloc(buffer, layout);

        let out = call_continuation_closure(cont_buffer, z);
        std::alloc::dealloc(cont_buffer, cont_layout);

        println!("x = {}, y = {}, z = {}", x, y, z);
        println!("(x - 1) + (y + x - 1) + x + y + z = ???");
        println!(
            "{} + {} + {} + {} + {} = {}",
            x - 1,
            y + x - 1,
            x,
            y,
            z,
            out
        );
    };
    // Exit code
    0
}

unsafe fn call_main_closure(closure_data_ptr: *const u8, y: i32) -> (*mut u8, Layout) {
    let size = size_Main_result() as usize;
    let layout = Layout::array::<u8>(size).unwrap();
    let buffer = std::alloc::alloc(layout) as *mut u8;

    // call_Main expects y to be stored in the flags.
    call_Main(
        // This flags pointer will never get dereferenced
        &y as *const i32,
        closure_data_ptr,
        buffer,
    );

    (buffer, layout)
}

unsafe fn call_continuation_closure(closure_data_ptr: *const u8, z: i32) -> i32 {
    let mut out: i32 = 0;

    // call_Continuation expects z to be stored in the flags.
    call_Continuation(
        // This flags pointer will never get dereferenced
        &z as *const i32,
        closure_data_ptr,
        &mut out as *mut i32,
    );

    out
}
