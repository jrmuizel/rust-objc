/*!
A Rust interface for Objective-C blocks.

For more information on the specifics of the block implementation, see
Clang's documentation: http://clang.llvm.org/docs/Block-ABI-Apple.html

# Invoking blocks

The `Block` struct is used for invoking blocks from Objective-C. For example,
consider this Objective-C function:

``` objc
int32_t sum(int32_t (^block)(int32_t, int32_t)) {
    return block(5, 8);
}
```

We could write it in Rust as the following:

```
use objc::block::Block;

fn sum(block: &mut Block<(i32, i32), i32>) -> i32 {
    block.call((5, 8))
}
```

Note the extra parentheses in the `call` method, since the arguments must be
passed as a tuple.

# Creating blocks

Creating a block to pass to Objective-C can be done with the `ConcreteBlock`
struct. For example, to create a block that adds two `i32`s, we could write:

```
use objc::block::ConcreteBlock;

let block = ConcreteBlock::new(|a: i32, b: i32| a + b);
let mut block = block.copy();
assert!(block.call((5, 8)) == 13);
```

It is important to copy your block to the heap (with the `copy` method) before
passing it to Objective-C; this is because our `ConcreteBlock` is only meant
to be copied once, and we can enforce this in Rust, but if Objective-C code
were to copy it twice we could have a double free.
*/

use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;
use libc::{c_int, c_ulong};

use runtime::{Class, Object};
use {EncodePtr, Id, Message};

#[link(name = "Foundation", kind = "framework")]
extern {
    static _NSConcreteStackBlock: Class;
}

/// Types that may be used as the arguments to an Objective-C block.
pub trait BlockArguments {
    /// Calls the given `Block` with self as the arguments.
    fn call_block<R>(self, block: &mut Block<Self, R>) -> R;
}

macro_rules! block_args_impl {
    ($($a:ident : $t:ident),*) => (
        impl<$($t),*> BlockArguments for ($($t,)*) {
            fn call_block<R>(self, block: &mut Block<Self, R>) -> R {
                let invoke: unsafe extern fn(*mut Block<Self, R> $(, $t)*) -> R = unsafe {
                    mem::transmute(block.invoke)
                };
                let ($($a,)*) = self;
                unsafe {
                    invoke(block $(, $a)*)
                }
            }
        }
    );
}

block_args_impl!();
block_args_impl!(a: A);
block_args_impl!(a: A, b: B);
block_args_impl!(a: A, b: B, c: C);
block_args_impl!(a: A, b: B, c: C, d: D);
block_args_impl!(a: A, b: B, c: C, d: D, e: E);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
block_args_impl!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);

/// An Objective-C block that takes arguments of `A` when called and
/// returns a value of `R`.
#[repr(C)]
pub struct Block<A, R> {
    isa: *const Class,
    flags: c_int,
    _reserved: c_int,
    invoke: unsafe extern fn(*mut Block<A, R>, ...) -> R,
}

// TODO: impl FnMut when it's possible
impl<A: BlockArguments, R> Block<A, R> where A: BlockArguments {
    /// Call self with the given arguments.
    pub fn call(&mut self, args: A) -> R {
        args.call_block(self)
    }
}

unsafe impl<A, R> Message for Block<A, R> { }

impl<A, R> EncodePtr for Block<A, R> {
    fn ptr_code() -> &'static str { "@?" }
}

pub trait IntoConcreteBlock<A, R> where A: BlockArguments {
    fn into_concrete_block(self) -> ConcreteBlock<A, R, Self>;
}

macro_rules! concrete_block_impl {
    ($f:ident) => (
        concrete_block_impl!($f,);
    );
    ($f:ident, $($a:ident : $t:ident),*) => (
        impl<$($t,)* R, X> IntoConcreteBlock<($($t,)*), R> for X
                where X: Fn($($t,)*) -> R {
            fn into_concrete_block(self) -> ConcreteBlock<($($t,)*), R, X> {
                unsafe extern fn $f<$($t,)* R, X>(
                        block_ptr: *mut ConcreteBlock<($($t,)*), R, X>
                        $(, $a: $t)*) -> R
                        where X: Fn($($t,)*) -> R {
                    let block = &*block_ptr;
                    (block.closure)($($a),*)
                }

                unsafe {
                    ConcreteBlock::with_invoke(
                        mem::transmute($f::<$($t,)* R, X>), self)
                }
            }
        }
    );
}

concrete_block_impl!(concrete_block_invoke_args0);
concrete_block_impl!(concrete_block_invoke_args1, a: A);
concrete_block_impl!(concrete_block_invoke_args2, a: A, b: B);
concrete_block_impl!(concrete_block_invoke_args3, a: A, b: B, c: C);
concrete_block_impl!(concrete_block_invoke_args4, a: A, b: B, c: C, d: D);
concrete_block_impl!(concrete_block_invoke_args5, a: A, b: B, c: C, d: D, e: E);
concrete_block_impl!(concrete_block_invoke_args6, a: A, b: B, c: C, d: D, e: E, f: F);
concrete_block_impl!(concrete_block_invoke_args7, a: A, b: B, c: C, d: D, e: E, f: F, g: G);
concrete_block_impl!(concrete_block_invoke_args8, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);
concrete_block_impl!(concrete_block_invoke_args9, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I);
concrete_block_impl!(concrete_block_invoke_args10, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J);
concrete_block_impl!(concrete_block_invoke_args11, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K);
concrete_block_impl!(concrete_block_invoke_args12, a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I, j: J, k: K, l: L);

/// An Objective-C block whose size is known at compile time and may be
/// constructed on the stack.
#[repr(C)]
pub struct ConcreteBlock<A, R, F> {
    base: Block<A, R>,
    descriptor: Box<BlockDescriptor<ConcreteBlock<A, R, F>>>,
    closure: F,
}

impl<A, R, F> ConcreteBlock<A, R, F>
        where A: BlockArguments, F: IntoConcreteBlock<A, R> {
    /// Constructs a `ConcreteBlock` with the given closure.
    /// When the block is called, it will return the value that results from
    /// calling the closure.
    pub fn new(closure: F) -> Self {
        closure.into_concrete_block()
    }
}

impl<A, R, F> ConcreteBlock<A, R, F> {
    /// Constructs a `ConcreteBlock` with the given invoke function and closure.
    /// Unsafe because the caller must ensure the invoke function takes the
    /// correct arguments.
    unsafe fn with_invoke(invoke: unsafe extern fn(*mut Self, ...) -> R,
            closure: F) -> Self {
        ConcreteBlock {
            base: Block {
                isa: &_NSConcreteStackBlock,
                // 1 << 25 = BLOCK_HAS_COPY_DISPOSE
                flags: 1 << 25,
                _reserved: 0,
                invoke: mem::transmute(invoke),
            },
            descriptor: Box::new(BlockDescriptor::new()),
            closure: closure,
        }
    }

    /// Copy self onto the heap.
    pub fn copy(self) -> Id<Block<A, R>> {
        unsafe {
            // The copy method is declared as returning an object pointer.
            let block: *mut Object = msg_send![&self.base, copy];
            let block = block as *mut Block<A, R>;
            // At this point, our copy helper has been run so the block will
            // be moved to the heap and we can forget the original block
            // because the heap block will drop in our dispose helper.
            mem::forget(self);
            Id::from_retained_ptr(block)
        }
    }
}

impl<A, R, F> Clone for ConcreteBlock<A, R, F> where F: Clone {
    fn clone(&self) -> Self {
        unsafe {
            ConcreteBlock::with_invoke(mem::transmute(self.invoke),
                self.closure.clone())
        }
    }
}

impl<A, R, F> Deref for ConcreteBlock<A, R, F> {
    type Target = Block<A, R>;

    fn deref(&self) -> &Block<A, R> {
        &self.base
    }
}

impl<A, R, F> DerefMut for ConcreteBlock<A, R, F> {
    fn deref_mut(&mut self) -> &mut Block<A, R> {
        &mut self.base
    }
}

unsafe extern fn block_context_dispose<B>(block: &mut B) {
    // Read the block onto the stack and let it drop
    ptr::read(block);
}

unsafe extern fn block_context_copy<B>(_dst: &mut B, _src: &B) {
    // The runtime memmoves the src block into the dst block, nothing to do
}

#[repr(C)]
struct BlockDescriptor<B> {
    _reserved: c_ulong,
    block_size: c_ulong,
    copy_helper: unsafe extern fn(&mut B, &B),
    dispose_helper: unsafe extern fn(&mut B),
}

impl<B> BlockDescriptor<B> {
    fn new() -> BlockDescriptor<B> {
        BlockDescriptor {
            _reserved: 0,
            block_size: mem::size_of::<B>() as c_ulong,
            copy_helper: block_context_copy::<B>,
            dispose_helper: block_context_dispose::<B>,
        }
    }
}

#[cfg(test)]
mod tests {
    use Id;
    use objc_test_utils;
    use super::{Block, ConcreteBlock};

    fn get_int_block_with(i: i32) -> Id<Block<(), i32>> {
        unsafe {
            let ptr = objc_test_utils::get_int_block_with(i);
            Id::from_retained_ptr(ptr as *mut _)
        }
    }

    fn get_add_block_with(i: i32) -> Id<Block<(i32,), i32>> {
        unsafe {
            let ptr = objc_test_utils::get_add_block_with(i);
            Id::from_retained_ptr(ptr as *mut _)
        }
    }

    fn invoke_int_block(block: &mut Block<(), i32>) -> i32 {
        let ptr = block as *mut _;
        unsafe {
            objc_test_utils::invoke_int_block(ptr as *mut _)
        }
    }

    fn invoke_add_block(block: &mut Block<(i32,), i32>, a: i32) -> i32 {
        let ptr = block as *mut _;
        unsafe {
            objc_test_utils::invoke_add_block(ptr as *mut _, a)
        }
    }

    #[test]
    fn test_call_block() {
        let mut block = get_int_block_with(13);
        assert!(block.call(()) == 13);
    }

    #[test]
    fn test_call_block_args() {
        let mut block = get_add_block_with(13);
        assert!(block.call((2,)) == 15);
    }

    #[test]
    fn test_create_block() {
        let mut block = ConcreteBlock::new(|| 13);
        let result = invoke_int_block(&mut block);
        assert!(result == 13);
    }

    #[test]
    fn test_create_block_args() {
        let mut block = ConcreteBlock::new(|a: i32| a + 5);
        let result = invoke_add_block(&mut block, 6);
        assert!(result == 11);
    }

    #[test]
    fn test_concrete_block_copy() {
        let s = "Hello!".to_string();
        let expected_len = s.len() as i32;
        let mut block = ConcreteBlock::new(move || s.len() as i32);
        assert!(invoke_int_block(&mut block) == expected_len);

        let mut copied = block.copy();
        assert!(invoke_int_block(&mut copied) == expected_len);
    }
}