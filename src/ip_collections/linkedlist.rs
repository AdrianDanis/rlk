//! LinkedList that creates links through provided memory

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
    tail: Option<NonNull<LLNode<T>>>,
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self { head: None, tail: None }
    }
}

// TOOD: rethink unsafe through this and parent mod interface
impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self { head: None, tail: None }
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
    unsafe fn set_prev(mut node: NonNull<LLNode<T>>, mut prev: Option<NonNull<LLNode<T>>>) {
        node.as_mut().as_mut().prev = prev;
        for it in prev.iter_mut() {
            it.as_mut().as_mut().next = Some(node);
        }
    }
    pub unsafe fn push_front(&mut self, item: Item<T>) {
        let data = LLData { prev: None, next: self.head };
        let node = LLNode::<T>::new(item.0, item.1, data);
        match self.head {
            None => self.tail = Some(node),
            Some(mut head) => head.as_mut().as_mut().prev = Some(node),
        }
        self.head = Some(node);
    }
    pub unsafe fn push_back(&mut self, item: Item<T>) {
        let data = LLData { prev: self.tail, next: None };
        let node = LLNode::<T>::new(item.0, item.1, data);
        match self.tail {
            None => self.head = Some(node),
            Some(mut tail) => tail.as_mut().as_mut().next = Some(node),
        }
        self.tail = Some(node);
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    unsafe fn unlink_node(&mut self, mut node: NonNull<LLNode<T>>) {
        let node = node.as_mut().as_mut();

        match node.prev {
            None => self.head = node.next,
            Some(mut prev) => prev.as_mut().as_mut().next = node.next,
        }

        match node.next {
            None => self.tail = node.prev,
            Some(mut next) => next.as_mut().as_mut().prev = node.prev,
        }
    }
}

impl<T: Clone + PartialEq> LinkedList<T> {
    pub unsafe fn remove(&mut self, value: T) -> Option<Item<T>> {
        if let Some(x) = self.head {
            let mut current = x;
            loop {
                if *current.as_mut().user_as_ref() == value {
                    self.unlink_node(current);
                    return Some(current.as_mut().as_item());
                }
                match current.as_mut().as_mut().next {
                    None => break,
                    Some(x) => current = x,
                }
            }
        }
        None
    }
}

impl<T: Clone + PartialOrd> LinkedList<T> {
    pub unsafe fn insert(&mut self, item: Item<T>) {
        let mut temp = Self::new();
        loop {
            match self.pop_front() {
                None => break,
                Some((slice, data)) => {
                    // See if we need to go beyond this element or not
                    if item.1 < data {
                        temp.push_back((slice, data));
                    } else {
                        self.push_front((slice, data));
                        break;
                    }
                },
            }
        }
        // now put our item in
        self.push_front(item);
        // put everything else back in. as we pushed back when inserting
        // we pop_front->push_front and end up with everything in the
        // original order
        loop {
            match temp.pop_front() {
                None => break,
                Some(item) => self.push_front(item),
            }
        }
    }
}

impl<T: Clone> LinkedList<T> {
    pub fn pop_front(&mut self) -> Option<Item<T>> {
        match self.head {
            None => None,
            Some(mut node) => {
                unsafe {
                    self.unlink_node(node);
                    Some(node.as_mut().as_item())
                }
            },
        }
    }
}
