pub mod hdr_encoder;

#[no_mangle]
pub extern "C" fn dinput(input: i32) -> i32 {
    println!("hello --from rust shared library");
    input * 2
}

#[no_mangle]
pub extern "C" fn set_num_threads(nthreads: i32) {
    use rayon::ThreadPoolBuilder;
    let tpb = ThreadPoolBuilder::new();
    let tpb = tpb.num_threads(nthreads as usize);
    tpb.build_global().unwrap();
}

#[no_mangle]
pub extern "C" fn print_hello_from_rust() {
    println!("Hello from Rust");
}

#[no_mangle]
pub extern "C" fn run_tmo(
    width: u32,
    height: u32,
    y: *mut u8,
    u: *const u8,
    v: *const u8,
    lum: *mut f32,
) {
    let length = (width * height) as usize;
    let y_slice = unsafe { std::slice::from_raw_parts_mut(y, length) };
    let u_slice = unsafe { std::slice::from_raw_parts(u, length) };
    let v_slice = unsafe { std::slice::from_raw_parts(v, length) };
    // let i_max = y_slice.iter().max().unwrap();
    // let i_min = y_slice.iter().min().unwrap();
    // println!("range before: {}, {}, {}", i_max, i_min, (*i_max as f32 / (*i_min as f32 + 0.0001)).log(10.0));
    let prev_lum = unsafe { *lum };
    let encoder = hdr_encoder::HdrEncoder::new(width, height, y_slice, u_slice, v_slice);
    let (new_y, curr_lum) = encoder.encode_v2(prev_lum);
    // println!("{}", new_y[34534]);
    // let i_max = new_y.iter().max().unwrap();
    // let i_min = new_y.iter().min().unwrap();
    // println!("range after: {}, {}, {}", i_max, i_min, (*i_max as f32 / (*i_min as f32 + 0.0001)).log(10.0));
    unsafe {
        let buffer = new_y.as_ptr();
        *lum = curr_lum;
        std::ptr::copy_nonoverlapping(buffer, y, length);
    }
}
