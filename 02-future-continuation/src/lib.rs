#![allow(non_snake_case)]

use core::alloc::Layout;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use libc;
use std::ffi::CStr;
use std::pin::Pin;
use std::os::raw::c_char;
use tokio::runtime::Runtime;
use std::time::Instant;
use std::future::Future;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut u8);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> i64;

    #[link_name = "roc__mainForHost_1_Fx_caller"]
    fn call_Fx(flags: *const u8, closure_data: *const u8, output: *mut u8);

    #[allow(dead_code)]
    #[link_name = "roc__mainForHost_1_Fx_size"]
    fn size_Fx() -> i64;

    #[link_name = "roc__mainForHost_1_Fx_result_size"]
    fn size_Fx_result() -> i64;
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
type FuturePtr = *mut (dyn Future<Output = i32> + Send);
type BoxFuture = Box<dyn Future<Output = i32> + Send>;

fn run_roc_main() -> BoxFuture {
    let size = unsafe { roc_main_size() } as usize;
    let layout = Layout::array::<u8>(size).unwrap();

    unsafe {
        // TODO allocate on the stack if it's under a certain size
        let buffer = std::alloc::alloc(layout);

        roc_main(buffer);

        let result = call_the_closure(buffer);

        std::alloc::dealloc(buffer, layout);

        result
    }
}

#[no_mangle]
pub extern "C" fn rust_main() -> i32 {
    unsafe {
        RT = MaybeUninit::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        RT.assume_init_ref().block_on(async {
            let n = 10;
            println!("Starting {} async tasks...", n);
            let mut handles = vec![];
            for i in 0..n {
                handles.push(tokio::spawn(async move {
                    let start = Instant::now();
                    Pin::from(run_roc_main()).await;
                    let elapsed_time = start.elapsed().as_millis();
                    println!("async task {} took {}ms and returned ???", i, elapsed_time);
                }));
            }
            futures::future::join_all(handles).await;
        });
    }
    // Exit code
    0
}

unsafe fn call_the_closure(closure_data_ptr: *const u8) -> BoxFuture {
    let size = size_Fx_result() as usize;
    let layout = Layout::array::<u8>(size).unwrap();
    let buffer = std::alloc::alloc(layout) as *mut u8;

    call_Fx(
        // This flags pointer will never get dereferenced
        MaybeUninit::uninit().as_ptr(),
        closure_data_ptr as *const u8,
        buffer as *mut u8,
    );

    let out = Box::from_raw(*(buffer as *mut FuturePtr));
    std::alloc::dealloc(buffer as *mut u8, layout);

    out
}


static mut DATA: i32 = 0;
#[no_mangle]
pub extern "C" fn roc_fx_readData() -> FuturePtr {
    use tokio::time::{sleep, Duration};
    Box::into_raw(Box::new(async {
        let x = unsafe{DATA};
        unsafe{DATA = x + 1;}
        sleep(Duration::from_millis(1000)).await;
        x
    }))
}