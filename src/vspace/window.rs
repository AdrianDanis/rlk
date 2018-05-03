// Thoughts on vspace windows

// Use windows with lifetimes for booting. Window into the physical address range has a
// lifetime of the window. Allocations from that window will therefore be dropped when
// the window ends. Allocations from the final kernel window have static. Allocations
// from the early window need some way to transfer to the final window, but this means
// they need an address change. is there a way to do that? 'create' a new object
// that happens to be the same as the original at a new address? Probably not, maybe
// a Copy needs to be done, but how to do that in place? I think Clone is sufficient
// as it implies the object can be 'memcpy'd to duplicate, so should also be safe
// to reinterpret at a new virtual address.
