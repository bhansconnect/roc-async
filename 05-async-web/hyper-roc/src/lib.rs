#![allow(non_snake_case)]

use std::convert::Infallible;
use std::ffi::{c_void, CStr};
use std::mem::MaybeUninit;
use std::os::raw::c_char;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use tokio::runtime::Runtime;

use roc_std::{RocResult, RocStr};

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(closure_data: *mut u8, req: *const Request<Body>);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> usize;

    #[link_name = "roc__mainForHost_1__Continuation_caller"]
    // The last field should be a pionter to a pionter, but we take it as a usize instead.
    fn call_Continuation(flags: *const u8, closure_data: *const u8, cont_ptr: *mut usize);

    #[link_name = "roc__mainForHost_1__Continuation_result_size"]
    fn call_Continuation_result_size() -> usize;

    #[link_name = "roc__mainForHost_1__DBRequestCont_caller"]
    fn call_DBRequestCont(flags: *const u64, closure_data: *const u8, output: *mut usize);

    #[link_name = "roc__mainForHost_1__DBRequestCont_result_size"]
    fn call_DBRequestCont_result_size() -> usize;

    #[link_name = "roc__mainForHost_1__LoadBodyCont_caller"]
    fn call_LoadBodyCont(
        flags: *const RocResult<RocStr, ()>,
        closure_data: *const u8,
        output: *mut usize,
    );

    #[link_name = "roc__mainForHost_1__LoadBodyCont_result_size"]
    fn call_LoadBodyCont_result_size() -> usize;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

static mut RT: MaybeUninit<Runtime> = MaybeUninit::uninit();

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
    libc::free(c_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    match tag_id {
        0 => {
            let slice = CStr::from_ptr(c_ptr as *const c_char);
            let string = slice.to_str().unwrap();
            println!("Roc hit a panic: {}", string);
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

#[repr(C)]
struct RocResponse {
    body: RocStr,
    status: u16,
}

#[inline(never)]
async fn fake_db_call(delay_ms: u64) -> u64 {
    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    // This is our dummy db call result.
    1
}

async fn root(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut resp = Response::new(Body::from(""));
    let mut cont_ptr: usize = 0;

    unsafe {
        let size = roc_main_size();
        stackalloc::alloca(size, |buffer| {
            roc_main(buffer.as_mut_ptr() as *mut u8, &req);

            call_Continuation(
                // This flags pointer will never get dereferenced
                MaybeUninit::uninit().as_ptr(),
                buffer.as_ptr() as *const u8,
                &mut cont_ptr,
            );
        });
        loop {
            match get_tag(cont_ptr) {
                0 => {
                    // DBRequest
                    let untagged_ptr = remove_tag(cont_ptr);
                    // We guarantee the delay is the first part of the tag.
                    // So we can just treate this as a pointer to the delay.
                    let delay_ms = *(untagged_ptr as *const u64);
                    let val = fake_db_call(delay_ms).await;
                    cont_ptr = call_DBRequestCont_closure(cont_ptr, val);
                }
                1 => {
                    // LoadBody
                    // We steal the mody and replace it with an empty body.
                    // Future calls to this method will get an empty string.
                    let mut tmp_body = Body::from("");
                    std::mem::swap(&mut tmp_body, req.body_mut());
                    let result = match hyper::body::to_bytes(tmp_body).await {
                        Ok(bytes) => RocResult::ok(RocStr::from_slice_unchecked(&bytes)),
                        _ => RocResult::err(()),
                    };
                    cont_ptr = call_LoadBodyCont_closure(cont_ptr, result);
                }
                2 => {
                    // Response
                    let out_ptr = remove_tag(cont_ptr) as *mut RocResponse;
                    *resp.status_mut() = StatusCode::from_u16((&*out_ptr).status)
                        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                    // TODO: Look into directly supporting RocStr here to avoid the copy.
                    *resp.body_mut() = Body::from((&*out_ptr).body.as_str().to_owned());
                    // Dropping doesn't work right with pointers to types.
                    // Work around that.
                    std::ptr::drop_in_place(out_ptr);
                    break;
                }
                _ => {
                    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    break;
                }
            }
        }
        deallocate_refcounted_tag(cont_ptr);
    }

    Ok(resp)
}

unsafe fn call_DBRequestCont_closure(args_and_data_ptr: usize, val: u64) -> usize {
    let closure_data_ptr = remove_tag(args_and_data_ptr + 16);
    let mut cont_ptr: usize = 0;

    call_DBRequestCont(
        &val,
        closure_data_ptr as *const u8,
        // buffer.as_mut_ptr() as *mut u8,
        &mut cont_ptr,
    );
    deallocate_refcounted_tag(args_and_data_ptr);

    // TODO: With nested continuations, this may need to be used.
    // Ran into issues related to it in the 04-nested-future-continuations
    // call_Continuation(
    //     // This flags pointer will never get dereferenced
    //     MaybeUninit::uninit().as_ptr(),
    //     buffer.as_ptr() as *const u8,
    //     &mut cont_ptr,
    // );

    cont_ptr
}

unsafe fn call_LoadBodyCont_closure(data_ptr: usize, result: RocResult<RocStr, ()>) -> usize {
    let closure_data_ptr = remove_tag(data_ptr);
    let mut cont_ptr: usize = 0;

    call_LoadBodyCont(
        &result,
        closure_data_ptr as *const u8,
        // buffer.as_mut_ptr() as *mut u8,
        &mut cont_ptr,
    );
    deallocate_refcounted_tag(data_ptr);

    // TODO: With nested continuations, this may need to be used.
    // Ran into issues related to it in the 04-nested-future-continuations
    // call_Continuation(
    //     // This flags pointer will never get dereferenced
    //     MaybeUninit::uninit().as_ptr(),
    //     buffer.as_ptr() as *const u8,
    //     &mut cont_ptr,
    // );

    cont_ptr
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    assert_eq!(
        unsafe { call_Continuation_result_size() },
        std::mem::size_of::<*const c_void>()
    );
    assert!(unsafe { call_DBRequestCont_result_size() } <= std::mem::size_of::<*const c_void>());
    assert!(unsafe { call_LoadBodyCont_result_size() } <= std::mem::size_of::<*const c_void>());
    unsafe {
        RT = MaybeUninit::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        RT.assume_init_ref().block_on(async {
            // For every connection, we must make a `Service` to handle all
            // incoming HTTP requests on said connection.
            let make_svc = make_service_fn(|_conn| {
                // This is the `Service` that will handle the connection.
                // `service_fn` is a helper to convert a function that
                // returns a Response into a `Service`.
                async { Ok::<_, Infallible>(service_fn(root)) }
            });
            let addr = ([0, 0, 0, 0], 3000).into();

            let server = Server::bind(&addr).serve(make_svc);

            println!("Listening on http://{}", addr);
            // Run this server for... forever!
            if let Err(e) = server.await {
                eprintln!("server error: {}", e);
            }
        });
    }
    // Exit code
    0
}

#[repr(C)]
pub enum RocMethod {
    Connect,
    Delete,
    Get,
    Head,
    Options,
    Other,
    Patch,
    Post,
    Put,
    Trace,
}

#[no_mangle]
pub extern "C" fn roc_fx_method(req: *const Request<Body>) -> RocMethod {
    match unsafe { &*req }.method() {
        &Method::CONNECT => RocMethod::Connect,
        &Method::DELETE => RocMethod::Delete,
        &Method::GET => RocMethod::Get,
        &Method::HEAD => RocMethod::Head,
        &Method::OPTIONS => RocMethod::Options,
        &Method::PATCH => RocMethod::Patch,
        &Method::POST => RocMethod::Post,
        &Method::PUT => RocMethod::Put,
        &Method::TRACE => RocMethod::Trace,
        _ => RocMethod::Other,
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_fx_path(req: *const Request<Body>) -> RocStr {
    RocStr::from_slice_unchecked((&*req).uri().path().as_bytes())
}

// TODO: make this work somehow?
// The issue is that we can't take ownership of the body to read it.
// #[no_mangle]
// pub unsafe extern "C" fn roc_fx_body(req_usize: usize) -> TraitObject {
//     use hyper::body::HttpBody;
//     let ptr: BodyFuturePtr = Box::into_raw(Box::new(async move {
//         let req = req_usize as *const Request<Body>;
//         match hyper::body::to_bytes((&*req).into_body().boxed()).await {
//             Ok(bytes) => RocResult::ok(RocStr::from_slice_unchecked(&bytes)),
//             _ => RocResult::err(()),
//         }
//     }));
//     unsafe { std::mem::transmute(ptr) }
// }

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
