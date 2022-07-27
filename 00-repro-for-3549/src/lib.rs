#![allow(non_snake_case)]

use std::ffi::{c_void, CStr};
use std::mem::MaybeUninit;
use std::os::raw::c_char;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut u8);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> usize;

    #[link_name = "roc__mainForHost_1__Continuation_caller"]
    // The last field should be a pionter to a pionter, but we take it as a usize instead.
    fn call_Continuation(flags: *const u8, closure_data: *const u8, cont_ptr: *mut usize);

    #[link_name = "roc__mainForHost_1__Continuation_result_size"]
    fn size_Continuation_result() -> usize;

    #[link_name = "roc__mainForHost_1__I32MoreCont_caller"]
    fn call_I32MoreCont(flags: *const i32, closure_data: *const u8, output: *mut usize);

    #[link_name = "roc__mainForHost_1__I32MoreCont_result_size"]
    fn size_I32MoreCont_result() -> usize;

    #[link_name = "roc__mainForHost_1__F32MoreCont_caller"]
    fn call_F32MoreCont(flags: *const f32, closure_data: *const u8, output: *mut usize);

    #[link_name = "roc__mainForHost_1__F32MoreCont_result_size"]
    fn size_F32MoreCont_result() -> usize;
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
    assert_eq!(
        unsafe { size_Continuation_result() },
        std::mem::size_of::<*const c_void>()
    );
    assert!(unsafe { size_I32MoreCont_result() } <= std::mem::size_of::<*const c_void>());
    assert!(unsafe { size_F32MoreCont_result() } <= std::mem::size_of::<*const c_void>());
    unsafe {
        let mut cont_ptr: usize = 0;
        let size = roc_main_size();
        stackalloc::alloca(size, |buffer| {
            roc_main(buffer.as_mut_ptr() as *mut u8);

            call_Continuation(
                // This flags pointer will never get dereferenced
                MaybeUninit::uninit().as_ptr(),
                buffer.as_ptr() as *const u8,
                &mut cont_ptr,
            );
        });
        let mut i = 10;
        let mut f = 10.0;
        loop {
            let untagged_ptr = remove_tag(cont_ptr);
            match get_tag(cont_ptr) {
                0 => {
                    // Done
                    let result = *(untagged_ptr as *const i32);
                    println!("Got result: {}", result);
                    break;
                }
                1 => {
                    // F32MoreCont
                    println!("Sending F32: {}", f);
                    cont_ptr = call_F32MoreCont_closure(cont_ptr, f);
                    f += 1.5;
                }
                2 => {
                    // I32MoreCont
                    println!("Sending I32: {}", i);
                    cont_ptr = call_I32MoreCont_closure(cont_ptr, i);
                    i += 2;
                }
                _ => {
                    panic!("invalid continuation tag");
                }
            }
        }
        deallocate_refcounted_tag(cont_ptr);
    }
    // Exit code
    0
}

unsafe fn call_I32MoreCont_closure(data_ptr: usize, val: i32) -> usize {
    let closure_data_ptr = remove_tag(data_ptr);
    let mut cont_ptr: usize = 0;

    call_I32MoreCont(&val, closure_data_ptr as *const u8, &mut cont_ptr);
    deallocate_refcounted_tag(data_ptr);

    cont_ptr
}

unsafe fn call_F32MoreCont_closure(data_ptr: usize, val: f32) -> usize {
    let closure_data_ptr = remove_tag(data_ptr);
    let mut cont_ptr: usize = 0;

    call_F32MoreCont(&val, closure_data_ptr as *const u8, &mut cont_ptr);
    deallocate_refcounted_tag(data_ptr);

    cont_ptr
}

unsafe fn deallocate_refcounted_tag(ptr: usize) {
    // TODO: handle this better.
    // To deallocate we first need to ignore the lower bits that include the tag.
    // Then we subtract 8 to get the refcount.
    let ptr_to_refcount = (remove_tag(ptr) - 8) as *mut c_void;
    roc_dealloc(ptr_to_refcount, 8);
}

fn get_tag(ptr: usize) -> u8 {
    ptr as u8 & 0x07
}

unsafe fn remove_tag(ptr: usize) -> usize {
    // TODO: is this correct always?
    ptr & 0xFFFF_FFFF_FFFF_FFF8
}
