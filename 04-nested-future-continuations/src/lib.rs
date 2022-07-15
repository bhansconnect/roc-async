#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]

use core::alloc::Layout;
use core::ffi::c_void;
use core::mem::MaybeUninit;
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

    #[link_name = "roc__mainForHost_1_Continuation_result_size"]
    fn size_Continuation_result() -> i64;

    #[link_name = "roc__mainForHost_1_Continuation_caller"]
    fn call_Cont(flags: *const u8, closure_data: *const u8, output: *mut u8);

    #[link_name = "roc__mainForHost_1_MoreCont_caller"]
    fn call_MoreCont(flags: *const i32, closure_data: *const u8, output: *mut *mut u8);

    #[link_name = "roc__mainForHost_1_MoreCont_size"]
    fn size_MoreCont() -> i64;

    #[link_name = "roc__mainForHost_1_MoreCont_result_size"]
    fn size_MoreCont_result() -> i64;
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
            let n = 20;
            println!("Roc + Tokio with an async host effect function");
            println!("Each task will grab a value that takes 1s +/- 50ms to load\n");
            println!("Starting {} async roc tasks on a single thread...", n);
            let mut handles = vec![];
            for i in 0..n {
                let start = Instant::now();
                handles.push(tokio::spawn(async move {
                    run_roc_main();
                    // let val = Pin::from(run_roc_main()).await;
                    // let out = call_continuation_closure(val);
                    // let elapsed_time = start.elapsed().as_millis();
                    // println!("async roc task {:2} took {:4}ms and returned {:3}", i, elapsed_time, out);
                }));
            }
            futures::future::join_all(handles).await;
        });
    }
    // Exit code
    0
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

type FuturePtr = *mut (dyn Future<Output = i32> + Send);
type BoxFuture = Box<dyn Future<Output = i32> + Send>;

fn run_roc_main() {
    let size = unsafe { roc_main_size() } as usize;
    let layout = Layout::array::<u8>(size).unwrap();

    unsafe {
        // TODO allocate on the stack if it's under a certain size
        let buffer = std::alloc::alloc(layout);

        roc_main(buffer);
        let cont_ptr = call_continuation_closure(buffer);
        std::alloc::dealloc(buffer, layout);

        dbg!(core::slice::from_raw_parts(remove_tag(cont_ptr), size_Continuation_result() as usize));
    }
}

unsafe fn call_continuation_closure(closure_data_ptr: *mut u8) -> *mut u8 {
    let size = size_Continuation_result() as usize;
    let layout = Layout::array::<u8>(size).unwrap();
    let buffer = std::alloc::alloc(layout) as *mut u8;

    call_Cont(
        MaybeUninit::uninit().as_ptr(),
        closure_data_ptr,
        buffer as *mut u8,
    );

    buffer
}
// unsafe fn call_morecont_closure(closure_data_ptr: *mut u8, val: i32) -> *mut u8 {
//     let mut buffer_ptr: *mut u8 = core::ptr::null::<u8>() as *mut u8;
//     // call_MoreCont expects val to be stored in the flags.
//     // Clear the tag from the closure data ptr before calling.
//     call_MoreCont(
//         &val as *const i32,
//         remove_tag(closure_data_ptr),
//         &mut buffer_ptr,
//     );

//     deallocate_refcounted_tag(closure_data_ptr);

//     buffer_ptr
// }

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



static mut DATA: i32 = 0;
#[no_mangle]
pub extern "C" fn roc_fx_readData() -> TraitObject {
    use tokio::time::{sleep, Duration};
    let ptr : FuturePtr = Box::into_raw(Box::new(async {
        use rand::{SeedableRng, Rng};
        let mut rng = rand::rngs::StdRng::from_entropy();
        let time = 1000 + rng.gen_range(-50..50);
        sleep(Duration::from_millis(time as u64)).await;
        let x = unsafe{DATA};
        unsafe{DATA = x + 1;}
        x
    }));
    unsafe {std::mem::transmute(ptr)}
}