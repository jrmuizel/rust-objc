use runtime::{Class, Method, Sel};
use {Encode, EncodeArguments};
use super::MessageError;

use objc_encode::{Encoding, Encodings, EncodingsIterateCallback};

pub fn verify_message_signature<A, R>(cls: &Class, sel: Sel)
        -> Result<(), MessageError>
        where A: EncodeArguments, R: Encode {
    let method = match cls.instance_method(sel) {
        Some(method) => method,
        None => return Err(MessageError(
            format!("Method {:?} not found on class {:?}",
                sel, cls)
        )),
    };

    let ret = R::encode();
    let expected_ret = method.return_type();
    if expected_ret != ret {
        return Err(MessageError(
            format!("Return type code {} does not match expected {} for method {:?}",
                ret, expected_ret, method.name())
        ));
    }

    // Add 2 for self and _cmd
    let count = 2 + A::len();
    let expected_count = method.arguments_count();
    if count != expected_count {
        return Err(MessageError(
            format!("Method {:?} accepts {} arguments, but {} were given",
                method.name(), expected_count, count)
        ));
    }

    let args = A::encodings();
    let mut comparator = MethodEncodingsComparator::new(method);
    args.each(&mut comparator);

    comparator.result
}

struct MethodEncodingsComparator<'a> {
    method: &'a Method,
    index: usize,
    result: Result<(), MessageError>,
}

impl<'a> MethodEncodingsComparator<'a> {
    fn new(method: &Method) -> MethodEncodingsComparator {
        MethodEncodingsComparator {
            method: method,
            // Start at 2 to skip self and _cmd
            index: 2,
            result: Ok(()),
        }
    }
}

impl<'a> EncodingsIterateCallback for MethodEncodingsComparator<'a> {
    fn call<E: ?Sized + Encoding>(&mut self, encoding: &E) -> bool {
        let index = self.index;
        self.index += 1;
        let expected = self.method.argument_type(index);
        if !expected.as_ref().map_or(false, |e| e == encoding) {
            let error = if let Some(expected) = expected {
                format!("Method {:?} expected argument at index {} with type code {} but was given {}",
                    self.method.name(), index, expected, encoding)
            } else {
                format!("Method {:?} expected no argument at index {} but was given {}",
                    self.method.name(), index, encoding)
            };
            self.result = Err(MessageError(error));
            // stop iteration
            true
        } else {
            // don't stop iteration
            false
        }
    }
}
