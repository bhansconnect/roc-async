#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use core::alloc::Layout;
use core::ffi::c_void;
use core::mem::MaybeUninit;
use log::{error, info};
use std::ffi::CStr;
use std::future::Future;
use std::os::raw::c_char;
use std::pin::Pin;
use std::time::Instant;
use tokio::runtime::Runtime;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(output: *mut u8, x: i32);

    #[link_name = "roc__mainForHost_size"]
    fn roc_main_size() -> i64;

    #[link_name = "roc__mainForHost_1_Continuation_caller"]
    fn call_Cont(flags: *const u8, closure_data: *const u8, output: *mut *mut u8);

    #[link_name = "roc__mainForHost_1_MoreCont_caller"]
    fn call_MoreCont(flags: *const i32, closure_data: *const u8, output: *mut *mut u8);
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
            error!("Roc hit a panic: {}", string);
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
    unsafe {
        RT = MaybeUninit::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
        );
        RT.assume_init_ref().block_on(async {
            let n = 10;
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .format_timestamp_millis()
                .init();
            info!("Roc + Tokio with an async host effect function");
            info!("Requesting data takes 1s +/- 50ms\n");
            info!("Starting {} async roc tasks on a single thread...", n);
            let mut handles = vec![];
            for i in 0..n {
                let task_kind = i % 3;
                info!("Roc task {:2}: starting with kind {}", i, task_kind);
                let start = Instant::now();
                handles.push(tokio::spawn(async move {
                    // TODO: The ergonomics are not great had to turn pointers into usize to avoid rust being angry.
                    let mut cont_ptr = run_roc_main(task_kind);
                    loop {
                        match get_tag(cont_ptr) {
                            0 => {
                                // Done
                                break;
                            }
                            1 => {
                                // MoreCont
                                info!("Roc task {:2}: requested more data", i);
                                let untagged_ptr = remove_tag(cont_ptr);
                                // We guarantee the future is the first part of the tag.
                                // So we can just treate this as a pointer to the future.
                                let box_future = Box::from_raw(*(untagged_ptr as *const FuturePtr));
                                let val = Pin::from(box_future).await;
                                info!("Roc task {:2}: was sent {}", i, val);
                                cont_ptr = call_morecont_closure(cont_ptr, val);
                            }
                            x => {
                                // Invalid
                                error!("got an invalid tag value: {}", x);
                                std::process::exit(2);
                            }
                        }
                    }
                    // load data from done
                    let out = *(remove_tag(cont_ptr) as *const i32);
                    let elapsed_time = start.elapsed().as_millis();
                    info!(
                        "Roc task {:2} took {:4}ms total and returned {:3}",
                        i, elapsed_time, out
                    );
                }));
            }
            futures::future::join_all(handles).await;
        });
    }
    // Exit code
    0
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

type FuturePtr = *mut (dyn Future<Output = i32> + Send);
type BoxFuture = Box<dyn Future<Output = i32> + Send>;

fn run_roc_main(x: i32) -> usize {
    let size = unsafe { roc_main_size() } as usize;
    let layout = Layout::array::<u8>(size).unwrap();

    unsafe {
        // TODO allocate on the stack if it's under a certain size
        let buffer = std::alloc::alloc_zeroed(layout);

        roc_main(buffer, x);
        let cont_ptr = call_continuation_closure(buffer);
        std::alloc::dealloc(buffer, layout);
        cont_ptr as usize
    }
}

unsafe fn call_continuation_closure(closure_data_ptr: *mut u8) -> *mut u8 {
    let mut buffer_ptr: *mut u8 = core::ptr::null::<u8>() as *mut u8;

    call_Cont(
        MaybeUninit::uninit().as_ptr(),
        closure_data_ptr,
        &mut buffer_ptr,
    );

    buffer_ptr
}
unsafe fn call_morecont_closure(future_and_data_ptr: usize, val: i32) -> usize {
    let mut buffer_ptr: *mut u8 = core::ptr::null::<u8>() as *mut u8;
    // call_MoreCont expects val to be stored in the flags.
    // Clear the tag from the closure data ptr before calling.
    let closure_data_ptr = remove_tag(future_and_data_ptr + 16);
    call_MoreCont(
        &val as *const i32,
        closure_data_ptr as *const u8,
        &mut buffer_ptr,
    );

    deallocate_refcounted_tag(future_and_data_ptr);

    buffer_ptr as usize
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

static mut DATA: i32 = 0;
#[no_mangle]
pub extern "C" fn roc_fx_readData() -> TraitObject {
    use tokio::time::{sleep, Duration};
    let ptr: FuturePtr = Box::into_raw(Box::new(async {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::from_entropy();
        let time = 1000 + rng.gen_range(-50..50);
        sleep(Duration::from_millis(time as u64)).await;
        let x = unsafe { DATA };
        unsafe {
            DATA = x + 1;
        }
        x
    }));
    unsafe { std::mem::transmute(ptr) }
}
