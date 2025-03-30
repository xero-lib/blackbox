#[derive(Default)]
pub struct RingBuff<T, const CAP: usize> {
    index: usize,
    saturated: bool,
    contents: Vec<T>,
    pub capacity: usize,
}

impl<T: Clone + Default, const CAP: usize> RingBuff<T, CAP> {
    pub fn new() -> Self {
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

// could do with some optimization
impl<T: Clone, const CAP: usize> RingBuff<T, CAP> {
    pub fn vectorize(&self) -> Vec<T> {
        [
            self.saturated.then(|| {
                self.contents[self.index..]
                    .iter()
                    .clone()
                    .map(T::to_owned)
                    .collect()
            }),
            self.index.ne(&0).then(|| {
                self.contents
                    .windows(self.index)
                    .next()
                    .unwrap_or_default()
                    .iter()
                    .clone()
                    .map(T::to_owned)
                    .collect::<Vec<T>>()
            }),
        ]
        .into_iter()
        .flatten()
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

    // pub fn push_slice(&mut self, values: &[T]) {
    //     let dist_to_end = (self.capacity - 1) - self.index;
    //     // does this help?
    //     // let values_len = values.len();

    //     if values.len() < dist_to_end {
    //         self.contents.splice(
    //             self.index..(self.index + values.len()),
    //             values.iter().map(T::to_owned),
    //         );
    //     } else {
    //         self.contents.splice(
    //             self.index..(self.capacity - 1), // shouldnt need to be capacity - 1
    //             values.iter().take(dist_to_end).map(T::to_owned),
    //         );
    //         self.contents.splice(
    //             0..values.len() - dist_to_end,
    //             values.iter().skip(dist_to_end).map(T::to_owned),
    //         );
    //     }

    //     self.increment_index(values.len());
    // }

    pub fn push_slice(&mut self, values: &[T])
    where
        T: Copy,
    {
        let dist_to_end = (self.capacity - 1) - self.index;
        // does this help?
        // let values_len = values.len();

        if values.len() < dist_to_end {
            self.contents[self.index..][..values.len()].copy_from_slice(values);
        } else {
            self.contents[self.index..self.capacity - 1].copy_from_slice(&values[..dist_to_end]);
            self.contents[0..values.len() - dist_to_end].copy_from_slice(&values[dist_to_end..]);
        }

        self.increment_index(values.len());
    }

    // could do with some optimization
    #[allow(unused)] //only for benchmarks
    pub fn push_slice_for(&mut self, values: &[T]) {
        let dist_to_end = (self.capacity - 1) - self.index;
        // does this help?
        // let values_len = values.len();

        if values.len() < dist_to_end {
            for i in self.index..(self.index + values.len()) {
                self.contents[i] = values[i - self.index].clone();
            }
        } else {
            for i in self.index..(self.capacity) {
                // shouldnt need to be -1
                self.contents[i] = values[i - self.index].clone();
            }

            for i in 0..(values.len() - dist_to_end) {
                self.contents[i] = values[i + dist_to_end].clone()
            }
        }

        self.increment_index(values.len());
    }
}
