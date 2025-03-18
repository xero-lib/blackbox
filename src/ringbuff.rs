#[derive(Default)]
pub struct RingBuff<T> {
    index: usize,
    saturated: bool,
    contents: Vec<T>,
    pub capacity: usize,
}

impl<T: Clone + Default> RingBuff<T> {
    pub fn new<const CAP: usize>() -> Self {
        // I don't really like this syntax. subject to change
        Self {
            capacity: CAP,
            contents: {
                let mut vec = Vec::<T>::with_capacity(CAP);
                vec.resize(CAP, T::default());
                vec
            },
            ..Default::default()
        }
    }
}

impl<T: Clone> RingBuff<T> {
    pub fn vectorize(&self) -> Vec<T> {
        [
            self.contents[..self.index]
                .iter()
                .clone()
                .map(T::to_owned)
                .collect::<Vec<T>>(),
            self.saturated
                .then(|| {
                    self.contents[self.index..]
                        .iter()
                        .clone()
                        .map(T::to_owned)
                        .collect()
                })
                .unwrap_or(Vec::new()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<T>>()
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

    pub fn push_slice(&mut self, values: &[T]) {
        let dist_to_end = (self.capacity - 1) - self.index;
        if values.len() < dist_to_end {
            self.contents.splice(
                self.index..(self.index + values.len()),
                values.iter().map(T::to_owned),
            );
        } else {
            self.contents.splice(
                self.index..(self.capacity - 1),
                values.iter().take(dist_to_end).map(T::to_owned),
            );
            self.contents.splice(
                0..values.len() - dist_to_end,
                values.iter().skip(dist_to_end).map(T::to_owned),
            );
        }

        self.increment_index(values.len());
    }
}
