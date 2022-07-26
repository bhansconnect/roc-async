use std::convert::Infallible;
use std::ffi::{c_void, CStr};
use std::mem::MaybeUninit;
use std::os::raw::c_char;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use tokio::runtime::Runtime;

use roc_std::RocStr;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut RocStr, req: *const Request<Body>);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> i64;

    // #[link_name = "roc__mainForHost_1_Continuation_caller"]
    // fn call_Cont(flags: *const u8, closure_data: *const u8, output: *mut *mut u8);

    // #[link_name = "roc__mainForHost_1_MoreCont_caller"]
    // fn call_MoreCont(flags: *const i32, closure_data: *const u8, output: *mut u8);

    // #[link_name = "roc__mainForHost_1_MoreCont_result_size"]
    // fn call_MoreCont_result_size() -> i64;
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

async fn root(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // let size = roc_main_size() as usize;
    // let layout = Layout::array::<u8>(size).unwrap();
    // let buffer = std::alloc::alloc_zeroed(layout);
    let mut out = RocStr::empty();

    unsafe {
        roc_main(&mut out, &req);
    }
    // let cont_ptr = call_continuation_closure(buffer);
    // std::alloc::dealloc(buffer, layout);
    // cont_ptr as usize

    // TODO: Look into directly supporting RocStr here to avoid the copy.
    Ok(Response::new(Body::from(out.as_str().to_owned())))
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
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
