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

    for dirent_result in fs::read_dir("ngtcp2/crypto/includes/ngtcp2").unwrap() {
        let dirent = dirent_result.unwrap();
        fs::copy(
            dirent.path(),
            include.join("ngtcp2").join(dirent.file_name()),
        )
        .unwrap();
    }

    let mut cfg = cc::Build::new();
    cfg.include("ngtcp2/lib/includes")
        .include(&include)
        .include("ngtcp2/crypto")
        .include("ngtcp2/lib")
        .file("ngtcp2/crypto/shared.c")
        .file("ngtcp2/lib/ngtcp2_acktr.c")
        .file("ngtcp2/lib/ngtcp2_addr.c")
        .file("ngtcp2/lib/ngtcp2_balloc.c")
        .file("ngtcp2/lib/ngtcp2_bbr.c")
        .file("ngtcp2/lib/ngtcp2_buf.c")
        .file("ngtcp2/lib/ngtcp2_callbacks.c")
        .file("ngtcp2/lib/ngtcp2_cc.c")
        .file("ngtcp2/lib/ngtcp2_cid.c")
        .file("ngtcp2/lib/ngtcp2_conn.c")
        .file("ngtcp2/lib/ngtcp2_conv.c")
        .file("ngtcp2/lib/ngtcp2_crypto.c")
        .file("ngtcp2/lib/ngtcp2_dcidtr.c")
        .file("ngtcp2/lib/ngtcp2_err.c")
        .file("ngtcp2/lib/ngtcp2_frame_chain.c")
        .file("ngtcp2/lib/ngtcp2_gaptr.c")
        .file("ngtcp2/lib/ngtcp2_idtr.c")
        .file("ngtcp2/lib/ngtcp2_ksl.c")
        .file("ngtcp2/lib/ngtcp2_log.c")
        .file("ngtcp2/lib/ngtcp2_map.c")
        .file("ngtcp2/lib/ngtcp2_mem.c")
        .file("ngtcp2/lib/ngtcp2_objalloc.c")
        .file("ngtcp2/lib/ngtcp2_opl.c")
        .file("ngtcp2/lib/ngtcp2_path.c")
        .file("ngtcp2/lib/ngtcp2_pcg.c")
        .file("ngtcp2/lib/ngtcp2_pkt.c")
        .file("ngtcp2/lib/ngtcp2_pmtud.c")
        .file("ngtcp2/lib/ngtcp2_ppe.c")
        .file("ngtcp2/lib/ngtcp2_pq.c")
        .file("ngtcp2/lib/ngtcp2_pv.c")
        .file("ngtcp2/lib/ngtcp2_qlog.c")
        .file("ngtcp2/lib/ngtcp2_range.c")
        .file("ngtcp2/lib/ngtcp2_ringbuf.c")
        .file("ngtcp2/lib/ngtcp2_rob.c")
        .file("ngtcp2/lib/ngtcp2_rst.c")
        .file("ngtcp2/lib/ngtcp2_rtb.c")
        .file("ngtcp2/lib/ngtcp2_settings.c")
        .file("ngtcp2/lib/ngtcp2_str.c")
        .file("ngtcp2/lib/ngtcp2_strm.c")
        .file("ngtcp2/lib/ngtcp2_transport_params.c")
        .file("ngtcp2/lib/ngtcp2_unreachable.c")
        .file("ngtcp2/lib/ngtcp2_vec.c")
        .file("ngtcp2/lib/ngtcp2_version.c")
        .file("ngtcp2/lib/ngtcp2_window_filter.c")
        .warnings(false)
        .define("NGHTTP3_STATICLIB", None)
        .define("HAVE_NETINET_IN", None)
        .define("HAVE_TIME_H", None)
        .out_dir(&lib);

    if cfg!(feature = "quictls") {
        cfg
            .define("HAVE_QUICTLS", None)
            .file("ngtcp2/crypto/quictls/quictls.c");

        if let Some(path) = env::var_os("DEP_QUICLTS_ROOT") {
            let path = PathBuf::from(path);
            cfg.include(path.join("include"));
        }
    }
    if cfg!(feature = "boringssl") {
        cfg
            .define("HAVE_BORINGSSL", None)
            .file("ngtcp2/crypto/boringssl/boringssl.c");
        if let Some(path) = env::var_os("DEP_BORINGSSL_ROOT") {
            let path = PathBuf::from(path);
            cfg.include(path.join("boringssl/src/include"));
        }
    }
    if cfg!(feature = "openssl") {
        cfg
            .define("HAVE_OPENSSL", None)
            .file("ngtcp2/crypto/ossl/ossl.c");
        if let Some(path) = env::var_os("DEP_OPENSSL_INCLUDE") {
            let path = PathBuf::from(path);
            cfg.include(path);
        }
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
