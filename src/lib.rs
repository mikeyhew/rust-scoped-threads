use std::thread;
use std::thread::JoinHandle;
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::boxed::Box;

pub struct Scope<'a> {
    join_handles: LinkedList<JoinHandle<()>>,
    _marker: PhantomData<::std::cell::Cell<&'a mut ()>>
}

impl<'a> Scope<'a> {
    pub fn spawn<F>(&mut self, f: F) where F: FnOnce() + Send + 'a {
        let f: Box<FnBox + Send + 'a> = Box::new(f);
        let f: Box<FnBox + Send + 'static> = unsafe{
            std::mem::transmute(f)
        };
        let join_handle = thread::spawn(move || f.call_box());
        self.join_handles.push_back(join_handle);
    }
}

pub fn scope<'a, F, R>(f: F) -> R where F: FnOnce(&mut Scope<'a>) -> R {
    let mut scope = Scope {
        join_handles: LinkedList::new(),
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
        scope(|mut scope| {
            scope.spawn(|| {
                foo = 1;
            });
        });
        assert!(foo == 1);
    }

    #[test]
    fn two_threads() {
        let mut foo = 0;
        let mut bar = 1;
        scope(|mut scope| {
            scope.spawn(|| {
                foo = 5;
            });
            scope.spawn(|| {
                bar = 7;
            })
        });
        assert!(foo == 5);
        assert!(bar == 7);
    }
}
