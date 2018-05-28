/// 'In Place' collections library
///
/// 'In Place' collections are datastructures based around forming data structures through
/// existing memory locations, instead of creating/allocating new objects

use core::ptr::NonNull;
use core::slice;
use core::mem::{size_of, align_of};

/// Generic node for in place data structures
///
/// Consists of a data type generally provided by the user, and a datatype for the data needed by
/// the collection
struct Node<U, C> {
    user: U,
    collection: C,
    /// Size of originally provided memory
    size: usize,
}

type Item<U> = (&'static mut[u8], U);

impl<U, C> Node<U, C> {
    fn check_mem(&self, mem: &[u8]) {
        if mem.len() < size_of::<Self>() {
            panic!("Insufficient memory to construct node");
        }
        if ((mem.as_ptr() as usize) % align_of::<Self>()) != 0 {
            panic!("Provided memory is not correctly aligned");
        }
    }
    pub unsafe fn new(mem: &'static mut [u8], user: U, collection: C) -> NonNull<Node<U, C>> {
        let node = mem.as_mut_ptr() as *mut Node<U, C>;
        *node = Self { user:user, collection:collection, size:mem.len() };
        NonNull::new(node).unwrap()
    }
    pub fn user_as_ref(&self) -> &U {
        &self.user
    }
    pub fn as_mut(&mut self) -> &mut C {
        &mut self.collection
    }
}

impl<U: Clone, C> Node<U, C> {
    pub unsafe fn as_item(&mut self) -> Item<U> {
        let v = self.user.clone();
        (slice::from_raw_parts_mut(self as *mut Self as *mut u8, self.size), v)
    }
}

mod linkedlist;

pub use self::linkedlist::LinkedList;
