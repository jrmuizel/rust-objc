/**
Registers a selector, returning a `Sel`.

# Example
```
# #[macro_use] extern crate objc;
# fn main() {
let sel = sel!(description);
let sel = sel!(setObject:forKey:);
# }
```
*/
#[cfg(not(feature = "static_sel"))]
#[macro_export]
macro_rules! sel {
    // Declare a function to hide unsafety, otherwise we can trigger the
    // unused_unsafe lint; see rust-lang/rust#8472
    ($name:ident) => ({
        #[inline(always)]
        fn register_sel(name_with_nul: &str) -> $crate::runtime::Sel {
            let ptr = name_with_nul.as_ptr() as *const _;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!(stringify!($name), '\0'))
    });
    ($($name:ident :)+) => ({
        #[inline(always)]
        fn register_sel(name_with_nul: &str) -> $crate::runtime::Sel {
            let ptr = name_with_nul.as_ptr() as *const _;
            unsafe { $crate::runtime::sel_registerName(ptr) }
        }
        register_sel(concat!($(stringify!($name), ':'),+, '\0'))
    });
}

/**
Registers a selector, returning a `Sel`.

# Example
```
# #[macro_use] extern crate objc;
# fn main() {
let sel = sel!(description);
let sel = sel!(setObject:forKey:);
# }
```
*/
#[cfg(feature = "static_sel")]
#[macro_export]
macro_rules! sel {
    // Declare a function to hide unsafety, otherwise we can trigger the
    // unused_unsafe lint; see rust-lang/rust#8472
    ($($t:tt)+) => ({
        #[inline(always)]
        fn do_it() -> $crate::runtime::Sel {

            // See sel-macros/macros.rs for implementation details.
            #[allow(dead_code)]
            #[derive(__objc_sel_internal)]
            struct X([(); (stringify!(__SEL_START_MARKER__ $($t)* __SEL_END_MARKER__), 0).1]);

            struct Cheaty(*const [u8; SEL_LEN]);
            unsafe impl Send for Cheaty {}
            unsafe impl Sync for Cheaty {}

            // Place the constant value in the correct section.
            #[link_section="__TEXT,__objc_methname,cstring_literals"]
            static VALUE : [u8; SEL_LEN] = SEL_DATA;
            #[link_section="__DATA,__objc_selrefs,literal_pointers,no_dead_strip"]
            static mut REF : Cheaty = Cheaty(&VALUE);

            // Produce a sel type as a result.
            // XXX(nika): Don't use transmute?
            unsafe { ::std::mem::transmute::<_, $crate::runtime::Sel>(REF.0) }
        }
        do_it()
    });
}

/**
Sends a message to an object.

The first argument can be any type that dereferences to a type that implements
`Message`, like a reference, pointer, or an `Id`.
The syntax is similar to the message syntax in Objective-C.
Variadic arguments are not currently supported.

# Example
``` no_run
# #[macro_use] extern crate objc;
# use objc::runtime::Object;
# fn main() {
# unsafe {
let obj: *mut Object;
# let obj: *mut Object = 0 as *mut Object;
let description: *const Object = msg_send![obj, description];
let _: () = msg_send![obj, setArg1:1 arg2:2];
# }
# }
```
*/
#[macro_export]
macro_rules! msg_send {
    (super($obj:expr, $superclass:expr), $name:ident) => ({
        let sel = sel!($name);
        match $crate::__send_super_message(&*$obj, $superclass, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    (super($obj:expr, $superclass:expr), $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        match $crate::__send_super_message(&*$obj, $superclass, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $name:ident) => ({
        let sel = sel!($name);
        match $crate::__send_message(&*$obj, sel, ()) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
    ($obj:expr, $($name:ident : $arg:expr)+) => ({
        let sel = sel!($($name:)+);
        match $crate::__send_message(&*$obj, sel, ($($arg,)*)) {
            Err(s) => panic!("{}", s),
            Ok(r) => r,
        }
    });
}
