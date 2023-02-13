use cfg_if::cfg_if;
cfg_if!{
    if #[cfg(all(feature = "audiopus-sys-backend", feature = "unsafe-libopus-backend"))] {
        compile_error!("feature \"audiopus-sys-backend\" and feature \"unsafe-libopus-backend\" cannot be enabled at the same time");
    } else if #[cfg(feature = "audiopus-sys-backend")] {
        pub use audiopus_sys as ffi;
        pub use libc::c_int as c_int;

        macro_rules! ctl {
            ($f:ident, $this:ident, $ctl:ident, $($rest:expr),*) => {
                match unsafe { ffi::$f($this.ptr, $ctl, $($rest),*) } {
                    code if code < 0 => return Err(Error::from_code(
                        concat!(stringify!($f), "(", stringify!($ctl), ")"),
                        code,
                    )),
                    _ => (),
                }
            }
        }

        pub(crate) use ctl;
    } else if #[cfg(feature = "unsafe-libopus-backend")] {
        #[allow(clippy::unsafe_removed_from_name)]
        pub use unsafe_libopus as ffi;
        #[allow(non_camel_case_types)]
        pub type c_int = i32;

        // defining C-style variadic functions in rust is unstable, so unsafe-libopus uses rust macros for its ctl functions
        macro_rules! ctl {
            ($f:ident, $this:ident, $ctl:ident, $($rest:expr),*) => {
                match unsafe { ffi::$f!($this.ptr, $ctl, $($rest),*) } {
                    code if code < 0 => return Err(Error::from_code(
                        concat!(stringify!($f), "(", stringify!($ctl), ")"),
                        code,
                    )),
                    _ => (),
                }
            }
        }

        pub(crate) use ctl;
    } else {
        compile_error!("either feature \"audiopus-sys-backend\" or feature \"unsafe-libopus-backend\" must be enabled");
    }

}
