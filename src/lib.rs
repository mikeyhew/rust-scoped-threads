use std::thread::JoinHandle;
use std::collections::LinkedList;

pub struct Scope {
  join_handles: LinkedList<JoinHandle<()>>,
}

pub fn scope<F, R>(f: F) -> R where F: FnOnce(&Scope) -> R {
  let mut scope = Scope { join_handles: LinkedList::new()};
  let ret = f(&mut scope);
  for join_handle in scope.join_handles {
    join_handle.join().unwrap();
  }
  ret
}


#[cfg(test)]
mod tests {
  use super::*;
    #[test]
    fn it_works() {
      let one = scope(|_|{1});
      assert!(one == 1);
    }
}
