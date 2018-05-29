//! 'In Place' collections library
//!
//! 'In Place' collections are datastructures based around forming data structures through
//! existing memory locations, instead of creating/allocating new objects

use core::ptr::NonNull;
use core::slice;
use core::mem::{size_of, align_of, transmute};

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

/// Common type used by functions for adding and removing items to in place data structures
type Item<U> = (&'static mut[u8], U);

impl<U, C> Node<U, C> {
    /// Checks that a slice is valid for using as a node
    ///
    /// Panics if the provided memory is not correct
    fn check_mem(mem: &[u8]) {
        if mem.len() < size_of::<Self>() {
            panic!("Insufficient memory to construct node");
        }
        if ((mem.as_ptr() as usize) % align_of::<Self>()) != 0 {
            panic!("Provided memory is not correctly aligned");
        }
    }
    /// Consume a slice of memory and produce a reference to a node
    ///
    /// This presents a safe interface as the provided slice is a mutable static and consumed
    ///
    /// # Panics
    ///
    /// The provided memory must have a sufficient size and alignment otherwise a panic is generated
    pub fn new(mem: &'static mut [u8], user: U, collection: C) -> NonNull<Node<U, C>> {
        Self::check_mem(mem);
        let node = mem.as_mut_ptr() as *mut Node<U, C>;
        unsafe {*node = Self { user:user, collection:collection, size:mem.len() }};
        NonNull::new(node).unwrap()
    }
    /// Retrieve a reference to the user data contained in the node
    pub fn user_as_ref(&self) -> &U {
        &self.user
    }
    /// Retrieve a mutable reference to the collection data contained in the node
    pub fn as_mut(&mut self) -> &mut C {
        &mut self.collection
    }
}

impl<U: Clone, C> Node<U, C> {
    /// Consume a node and retrieve the contained data and original slice
    pub fn as_item(&mut self) -> Item<U> {
        let v = self.user.clone();
        unsafe {(slice::from_raw_parts_mut(self as *mut Self as *mut u8, self.size), v)}
    }
}

mod linkedlist;

pub use self::linkedlist::LinkedList;
