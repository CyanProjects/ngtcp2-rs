extern crate libngtcp2_sys as ffi;

fn main() {
    unsafe {
        ffi::ngtcp2_version(0);
    }
}
