/// LinkedList that creates links through provided memory

use core::ptr::NonNull;
use core::mem::{size_of,align_of};

use super::{Node, Item};

type LLNode<T> = Node<T, LLData<T>>;

struct LLData<T> {
    next: Option<NonNull<LLNode<T>>>,
    prev: Option<NonNull<LLNode<T>>>,
}

impl<T> Default for LLData<T> {
    fn default() -> Self {
        Self { next: None, prev: None }
    }
}

pub struct LinkedList<T> {
    head: Option<NonNull<LLNode<T>>>,
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self { head: None }
    }
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self { head: None }
    }
    pub const fn size_of_node(&self) -> usize {
        size_of::<LLNode<T>>()
    }
    pub const fn align_of_node(&self) -> usize {
        align_of::<LLNode<T>>()
    }
    unsafe fn set_next(mut node: NonNull<LLNode<T>>, mut next: Option<NonNull<LLNode<T>>>) {
        node.as_mut().as_mut().next = next;
        for it in next.iter_mut() {
            it.as_mut().as_mut().prev = Some(node);
        }
    }
    pub unsafe fn push_front(&mut self, item: Item<T>) {
        // Build a node and get a reference to it
        let node = LLNode::<T>::new(item.0, item.1, LLData::<T>::default());
        Self::set_next(node, self.head);
        self.head = Some(node);
    }
    pub fn remove(&mut self, value: T) -> Option<Item<T>> {
        unimplemented!()
    }
    pub unsafe fn insert(&mut self, item: Item<T>) {
        unimplemented!()
    }
    pub fn pop_front(&mut self) -> Option<Item<T>> {
        unimplemented!()
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

