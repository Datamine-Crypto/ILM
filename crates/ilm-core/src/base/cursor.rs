use patterns::{because, source};

pub struct ItemCursor;
source!(ItemCursor, "a reader of a list of items that goes from left to right one place at a time and looks at the item at its place before it takes it");

pub trait ItemReader<T: Copy> {
    fn items_at(&self) -> (&[T], usize);

    fn peek(&self) -> Option<T> {
        let (items, at) = self.items_at();
        items.get(at).copied()
    }
}
because!(ItemReader, ItemCursor, "a reader of a list of items with its place: a reader gives its items with its place, and peek gives the item at the place, or nothing at the end");
