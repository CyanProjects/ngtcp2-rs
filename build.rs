extern crate cc;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let lib_version = env::var("CARGO_PKG_VERSION")
        .unwrap()
        .split('+')
        .nth(1)
        .unwrap()
        .to_string();
    let major = lib_version
        .split('.')
        .nth(0)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let minor = lib_version
        .split('.')
        .nth(1)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let patch = lib_version
        .split('.')
        .nth(2)
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let ver = fs::read_to_string("ngtcp2/lib/includes/ngtcp2/version.h.in")
        .unwrap()
        .replace("@PACKAGE_VERSION@", &lib_version)
        .replace(
            "@PACKAGE_VERSION_NUM@",
            &format!("0x{:02x}{:02x}{:02x}", major, minor, patch),
        );

    let install = out_dir.join("i");

    let include = install.join("include");
    let lib = install.join("lib");
    let pkgconfig = lib.join("pkgconfig");
    fs::create_dir_all(include.join("ngtcp2")).unwrap();
    fs::create_dir_all(&pkgconfig).unwrap();
    fs::write(include.join("ngtcp2/version.h"), ver).unwrap();
    fs::copy(
        "ngtcp2/lib/includes/ngtcp2/ngtcp2.h",
        include.join("ngtcp2/ngtcp2.h"),
    )
        .unwrap();

    let mut cfg = cc::Build::new();
    cfg.include("ngtcp2/lib/includes")
        .include(&include)
        .file("ngtcp2/lib/ngtcp2_version.c")
        .warnings(false)
        .define("NGHTTP3_STATICLIB", None)
        .define("HAVE_NETINET_IN", None)
        .define("HAVE_TIME_H", None)
        .out_dir(&lib);

    if cfg!(feature = "quictls") {
        cfg.define("HAVE_QUICTLS", None);
    }

    if target.contains("windows") {
        // Apparently MSVC doesn't have `ssize_t` defined as a type
        if target.contains("msvc") {
            match &env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap()[..] {
                "64" => {
                    cfg.define("ssize_t", "int64_t");
                }
                "32" => {
                    cfg.define("ssize_t", "int32_t");
                }
                s => panic!("unknown pointer size: {}", s),
            }
        }
    } else {
        cfg.define("HAVE_ARPA_INET_H", None);
    }
    cfg.compile("ngtcp2");

    println!("cargo:root={}", install.display());

    let pc = fs::read_to_string("ngtcp2/lib/libngtcp2.pc.in")
        .unwrap()
        .replace("@prefix@", install.to_str().unwrap())
        .replace("@exec_prefix@", "")
        .replace("@libdir@", lib.to_str().unwrap())
        .replace("@includedir@", include.to_str().unwrap())
        .replace("@VERSION@", &lib_version);
    fs::write(pkgconfig.join("libngtcp2.pc"), pc).unwrap();
}
