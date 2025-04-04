#[derive(Default)]
pub struct RingBuff<T> {
    index: usize,
    saturated: bool,
    contents: Vec<T>,
    pub capacity: usize,
}

impl<T: Clone + Copy + Default> RingBuff<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            capacity: cap,
            contents: {
                let mut vec = Vec::with_capacity(cap);
                vec.resize(cap, T::default());
                vec
            },
            ..Default::default()
        }
    }
}

impl<T: Clone> RingBuff<T> {
    pub fn vectorize(&self) -> Vec<T> {
        if self.saturated {
            self.contents[self.index..].iter().cloned().chain(self.contents[..self.index].iter().cloned()).collect()
        } else {
            self.contents[..self.index].to_vec()
        }
    }

    // don't you dare commit this
    fn increment_index(&mut self, by: usize) {
        let dist_to_end = (self.capacity - 1) - self.index;

        if by > dist_to_end {
            self.index = (by - 1) - dist_to_end;
            self.saturated = true;
        } else {
            self.index += by;
        }
    }

    #[allow(unused)]
    pub fn push(&mut self, value: T) {
        self.contents[self.index] = value;
        self.increment_index(1);
    }

    pub fn push_slice(&mut self, values: &[T])
    where T: Copy
    {
        let dist_to_end = (self.capacity - 1) - self.index;

        if values.len() < dist_to_end {
            self.contents[self.index..][..values.len()].copy_from_slice(values);
        } else {
            self.contents[self.index..self.capacity - 1].copy_from_slice(&values[..dist_to_end]);
            self.contents[..values.len() - dist_to_end ].copy_from_slice(&values[dist_to_end..]);
        }

        self.increment_index(values.len());
    }
}
