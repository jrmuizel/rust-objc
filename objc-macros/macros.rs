extern crate proc_macro;

use std::fmt::Write;
use proc_macro::TokenStream;

// Static Selectors:
//
// In objc, method calls etc. perform efficient lookups with interned strings.
// Rather than computing these for each call, the linker provides a solution for
// link-time sel deduplication. We store the strings in a specific section, and
// our references are automatically deduplicated.
//
// Unfortunately, we need our string length and null-terminated value as a
// constant to declare the array literal type. So we need to know the length of
// our string literal. To do this, we use this custom derive.
//
// '__objc_sel_internal' converts the tokens between '__SEL_START_MARKER__' and
// '__SEL_END_MARKER__' into a string, counts the number of bytes, and generates
// constant declarations for both the length (SEL_LEN) and data (SEL_DATA).
//
// The sel!() macro then uses these constants to declare the data in the correct
// sections, and get efficient selectors.

const SELSTART: &str = "__SEL_START_MARKER__";
const SELEND: &str = "__SEL_END_MARKER__";

#[proc_macro_derive(__objc_sel_internal)]
pub fn sel_internal(ts: TokenStream) -> TokenStream {
    let tsbuf = ts.to_string();

    // Use markers to find the start and end of useful data.
    let start = tsbuf.find(SELSTART).unwrap() + SELSTART.len();
    let end = tsbuf.rfind(SELEND).unwrap();
    let body = s[start..end].trim();

    // Create the data literal & count the byte length.
    let mut len = 0;
    let mut data = String::new();
    for byte in tostore.chars().filter(|c| !c.is_whitespace()) {
        len += 1;
        write!(&mut arraylit, "{}, ", byte).unwrap();
    }

    // These length & data constants are used by the sel! macro.
    format!("\
        const SEL_LEN: usize = {} + 1;\
        const SEL_DATA: [u8; SEL_LEN] = [{}0];",
        len, data).parse().unwrap()
}
