use crate::conn::ConnHandle;

pub struct Cursor<T> {
    items: Vec<T>,
    cursor: usize,
}

impl<T> Cursor<T> {
    pub fn new(items: Vec<T>) -> Self {
        Cursor { items, cursor: 0 }
    }

    pub fn next(&mut self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        let item = &self.items[self.cursor];
        self.cursor = (self.cursor + 1) % self.items.len();

        Some(item)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }
}
