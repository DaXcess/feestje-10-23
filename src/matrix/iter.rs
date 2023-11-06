use embedded_graphics::pixelcolor::Rgb888;

pub struct LoopingIterator<'a> {
    items: &'a Vec<Rgb888>,
    current_index: usize,
}

impl<'a> LoopingIterator<'a> {
    pub fn new(items: &'a Vec<Rgb888>) -> Self {
        Self {
            items,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for LoopingIterator<'a> {
    type Item = Rgb888;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        let item = &self.items[self.current_index];
        self.current_index = (self.current_index + 1) % self.items.len();
        Some(*item)
    }
}

impl<'a> DoubleEndedIterator for LoopingIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        if self.current_index == 0 {
            self.current_index = self.items.len() - 1;
        } else {
            self.current_index -= 1;
        }

        let item = &self.items[self.current_index];
        Some(*item)
    }
}

pub trait IterLooping {
    fn iter_looping(&self) -> LoopingIterator;
}

impl IterLooping for Vec<Rgb888> {
    fn iter_looping(&self) -> LoopingIterator {
        LoopingIterator::new(self)
    }
}
