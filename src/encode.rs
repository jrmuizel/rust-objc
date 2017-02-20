use std::os::raw::c_char;

use malloc_buf::Malloc;
use objc_encode::{Encode, Encodings};
use objc_encode::encoding::Primitive;
use objc_encode::parse::StrEncoding;

use runtime::{Class, Object, Sel};

pub type MallocEncoding = StrEncoding<Malloc<str>>;

pub unsafe fn from_malloc_str(ptr: *mut c_char) -> MallocEncoding {
    let buf = Malloc::from_c_str(ptr);
    StrEncoding::new_unchecked(buf.unwrap())
}

unsafe impl Encode for Sel {
    type Encoding = Primitive;
    fn encode() -> Primitive { Primitive::Sel }
}

unsafe impl<'a> Encode for &'a Object {
    type Encoding = Primitive;
    fn encode() -> Primitive { Primitive::Object }
}

unsafe impl<'a> Encode for &'a mut Object {
    type Encoding = Primitive;
    fn encode() -> Primitive { Primitive::Object }
}

unsafe impl<'a> Encode for &'a Class {
    type Encoding = Primitive;
    fn encode() -> Primitive { Primitive::Class }
}

unsafe impl<'a> Encode for &'a mut Class {
    type Encoding = Primitive;
    fn encode() -> Primitive { Primitive::Class }
}

/// Types that represent a group of arguments, where each has an Objective-C
/// type encoding.
pub trait EncodeArguments {
    /// The type as which the encodings for Self will be returned.
    type Encs: Encodings;

    /// Returns the Objective-C type encodings for Self.
    fn encodings() -> Self::Encs;

    /// Returns the number of encodings in Self.
    fn len() -> usize;
}

macro_rules! count_idents {
    () => (0);
    ($a:ident) => (1);
    ($a:ident, $($b:ident),+) => (1 + count_idents!($($b),*));
}

macro_rules! encode_args_impl {
    ($($t:ident),*) => (
        impl<$($t: Encode),*> EncodeArguments for ($($t,)*) {
            type Encs = ($(<$t as Encode>::Encoding,)*);

            fn encodings() -> Self::Encs {
                ($($t::encode(),)*)
            }

            fn len() -> usize {
                count_idents!($($t),*)
            }
        }
    );
}

encode_args_impl!();
encode_args_impl!(A);
encode_args_impl!(A, B);
encode_args_impl!(A, B, C);
encode_args_impl!(A, B, C, D);
encode_args_impl!(A, B, C, D, E);
encode_args_impl!(A, B, C, D, E, F);
encode_args_impl!(A, B, C, D, E, F, G);
encode_args_impl!(A, B, C, D, E, F, G, H);
encode_args_impl!(A, B, C, D, E, F, G, H, I);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J, K);
encode_args_impl!(A, B, C, D, E, F, G, H, I, J, K, L);

#[cfg(test)]
mod tests {
    use objc_encode::Encode;
    use runtime::{Class, Object, Sel};

    #[test]
    fn test_encode() {
        assert!(<&Object>::encode().to_string() == "@");
        assert!(<*mut Object>::encode().to_string() == "@");
        assert!(<&Class>::encode().to_string() == "#");
        assert!(Sel::encode().to_string() == ":");
    }
}
