#![allow(non_snake_case)]

use core::alloc::Layout;
use core::ffi::c_void;
use libc;
use std::ffi::CStr;
use std::os::raw::c_char;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut *mut u8);

    #[link_name = "roc__mainForHost_1_Continuation_result_size"]
    fn size_Continuation_result() -> i64;

    #[link_name = "roc__mainForHost_1_MoreCont_caller"]
    fn call_MoreCont(flags: *const i32, closure_data: *const u8, output: *mut *mut u8);

    #[link_name = "roc__mainForHost_1_MoreCont_size"]
    fn size_MoreCont() -> i64;

    #[link_name = "roc__mainForHost_1_MoreCont_result_size"]
    fn size_MoreCont_result() -> i64;
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
    let data = [21, 15, -9, 12, 32];
    unsafe {
        let mut cont_ptr: *mut u8 = core::ptr::null::<u8>() as *mut u8;

        roc_main(&mut cont_ptr);

        let mut i = 0;
        // Note: zero only works here cause tag zero happens to be Done.
        while get_tag(cont_ptr) != 0 {
            println!("Roc wants more data. Passing in {}.", data[i]);
            cont_ptr = call_morecont_closure(cont_ptr, data[i]);
            i += 1;
        }

        let out = *(remove_tag(cont_ptr) as *const i32);
        println!("Roc produced an output of {}.", out);
    };
    // Exit code
    0
}

unsafe fn call_morecont_closure(closure_data_ptr: *mut u8, val: i32) -> *mut u8 {
    let mut buffer_ptr: *mut u8 = core::ptr::null::<u8>() as *mut u8;
    // call_MoreCont expects val to be stored in the flags.
    // Clear the tag from the closure data ptr before calling.
    call_MoreCont(
        &val as *const i32,
        remove_tag(closure_data_ptr),
        &mut buffer_ptr,
    );

    deallocate_refcounted_tag(closure_data_ptr);

    buffer_ptr
}

unsafe fn deallocate_refcounted_tag<T>(ptr: *mut T) {
    // TODO: handle this better.
    // To deallocate we first need to ignore the lower bits that inclued the tag.
    // Then we subtract 8 to get the refcount.
    let ptr_to_refcount = remove_tag(ptr).offset(-8) as *mut c_void;
    roc_dealloc(ptr_to_refcount, 8);
}

fn get_tag<T>(ptr: *const T) -> u8 {
    ptr as u8 & 0x07
}

unsafe fn remove_tag<T>(ptr: *mut T) -> *mut T {
    // TODO: is this correct always?
    (ptr as usize & 0xFFFF_FFFF_FFFF_FFF8) as *mut T
}
