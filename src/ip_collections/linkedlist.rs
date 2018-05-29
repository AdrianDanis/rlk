//! LinkedList that creates links through provided memory

use core::ptr::NonNull;
use core::mem::{size_of,align_of};

use super::{Node, Item};

type LLNode<T> = Node<T, LLData<T>>;

/// Doubly linked list through provided memory
///
/// Compared to a normal `LinkedList` this supports removal and insertion into the middle of
/// the list.
pub struct LinkedList<T> {
    head: Option<NonNull<LLNode<T>>>,
    tail: Option<NonNull<LLNode<T>>>,
}

struct LLData<T> {
    next: Option<NonNull<LLNode<T>>>,
    prev: Option<NonNull<LLNode<T>>>,
}

impl<T> Default for LLData<T> {
    fn default() -> Self {
        Self { next: None, prev: None }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self { head: None, tail: None }
    }
}

impl<T> LinkedList<T> {
    /// Creates an empty `LinkedList<T>`
    pub const fn new() -> Self {
        Self { head: None, tail: None }
    }
    /// Minimum size of memory that can be inserted into the `LinkedList`
    pub const fn size_of_node(&self) -> usize {
        size_of::<LLNode<T>>()
    }
    /// Minimum alignment of memory that can be inserted into the `LinkedList`
    pub const fn align_of_node(&self) -> usize {
        align_of::<LLNode<T>>()
    }
    /// Push an item to the front of the list
    ///
    /// The provided `Item` is consumed and must respect `size_of_node` and `align_of_node`
    pub fn push_front(&mut self, item: Item<T>) {
        unsafe {
            let data = LLData { prev: None, next: self.head };
            let node = LLNode::<T>::new(item.0, item.1, data);
            match self.head {
                None => self.tail = Some(node),
                Some(mut head) => head.as_mut().as_mut().prev = Some(node),
            }
            self.head = Some(node);
        }
    }
    /// Push an item to the back of the list
    ///
    /// The provided `Item` is consumed and must respect `size_of_node` and `align_if_node`
    pub fn push_back(&mut self, item: Item<T>) {
        unsafe {
            let data = LLData { prev: self.tail, next: None };
            let node = LLNode::<T>::new(item.0, item.1, data);
            match self.tail {
                None => self.head = Some(node),
                Some(mut tail) => tail.as_mut().as_mut().next = Some(node),
            }
            self.tail = Some(node);
        }
    }
    /// Returns whether the `LinkedList` is empty or not
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl<T> LinkedList<T> {
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
    /// Search for a node in the `LinkedList` and remove it if found
    ///
    /// The node is searched for by comparing against the provided sample user data
    pub fn remove(&mut self, value: T) -> Option<Item<T>> {
        unsafe {
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
}

impl<T: Clone + PartialOrd> LinkedList<T> {
    /// Insert a new item into a sorted `LinkedList`
    ///
    /// The provided `Item` is inserted before the first element found, by walking from the head,
    /// that compares as greater than the provided `Item`. Using `insert` only makes sense if
    /// `insert` is used to insert all the nodes into the `LinkedList` or you otherwise know that
    /// the list is sorted.
    ///
    /// The provided `Item` is consumed and must respect `size_of_node` and `align_of_node`
    pub fn insert(&mut self, item: Item<T>) {
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
    /// Remove the front node in the `LinkedList` and return it
    ///
    /// Returns `None` if the `LinkedList` is empty
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
