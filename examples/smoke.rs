extern crate libngtcp2_sys as ffi;

fn main() {
    unsafe {
        ffi::ngtcp2_conn_del(std::ptr::null_mut());
        ffi::ngtcp2_version(0);
    }
}
