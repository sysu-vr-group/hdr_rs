pub mod hdr_encoder;

#[no_mangle]
pub extern "C" fn dinput(input: i32) -> i32 {
    println!("hello --from rust shared library");
    input * 2
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

    let prev_lum = unsafe { *lum };
    let encoder = hdr_encoder::HdrEncoder::new(width, height, y_slice, u_slice, v_slice);
    let (new_y, curr_lum) = encoder.encode(prev_lum);
    // println!("{}", new_y[34534]);
    unsafe {
        let buffer = new_y.as_ptr();
        *lum = curr_lum;
        std::ptr::copy_nonoverlapping(buffer, y, length);
    }
}
