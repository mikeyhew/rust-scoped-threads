use std::thread;
use std::thread::JoinHandle;
use std::marker::PhantomData;
use std::{ptr, mem};

#[macro_use]
extern crate stack_ptr;
use stack_ptr::StackPtr;

pub struct Scope<'a> {
    join_handles: Vec<JoinHandle<()>>,
    _marker: PhantomData<::std::cell::Cell<&'a mut ()>>
}

/// Use this type alias so we can call `coerce_stackptr!``. Rust versions < 1.18 don't support `+` in type arguments to macros.
type SendableFn<'a> = FnOnceUnsafe + Send + 'a;

impl<'a> Scope<'a> {
    pub fn spawn<F>(&mut self, f: StackPtr<'a, F>) where F: FnOnce() + Send + 'a,  {
        let f = coerce_stackptr!(f, SendableFn<'a>);
        let mut f: StackPtr<SendableFn<'static>> = unsafe{
            mem::transmute(f)
        };
        let join_handle = thread::spawn(move || {
            unsafe { f.call_once_unsafe(); }
            mem::forget(f);
        });
        self.join_handles.push(join_handle);
    }
}

pub fn scope<'a, F, R>(f: F) -> R where F: FnOnce(&mut Scope<'a>) -> R {
    let mut scope = Scope {
        join_handles: Vec::new(),
        _marker: PhantomData
    };
    let ret = f(&mut scope);
    for join_handle in scope.join_handles {
        join_handle.join().unwrap();
    }
    ret
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) { (*self)() }
}

/// Calls FnOnce::call_once() on self. You need to call std::mem::forget on self afterward.
trait FnOnceUnsafe {
    unsafe fn call_once_unsafe(&mut self);
}

impl<'a, F: FnOnce()> FnOnceUnsafe for F {
    unsafe fn call_once_unsafe(&mut self) { ptr::read(self)() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let one = scope(|_|{1});
        assert!(one == 1);
    }

    #[test]
    fn one_thread() {
        let mut foo = 0;
        {
            stack_ptr! {
                let closure: StackPtr<_> = StackPtr::new(||{
                    foo = 1;
                });
            }

            scope(|mut scope| {
                scope.spawn(closure);
            });
        }
        assert!(foo == 1);
    }

    #[test]
    fn two_threads() {
        let mut foo = 0;
        let mut bar = 1;
        {
            stack_ptr! {
                let closure1: StackPtr<_> = StackPtr::new(||{
                    foo = 5
                });
            }

            stack_ptr! {
                let closure2: StackPtr<_> = StackPtr::new(||{
                    bar = 7
                });
            }

            scope(|mut scope| {
                scope.spawn(closure1);
                scope.spawn(closure2);
            });
        }
        assert!(foo == 5);
        assert!(bar == 7);
    }
}
