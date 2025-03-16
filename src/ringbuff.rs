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

    pub fn push(&mut self, value: T) {
        self.contents[self.index] = value;
        if self.index == self.capacity - 1 {
            self.index = 0;
            self.saturated = true;
        } else {
            self.index += 1;
        }
    }
}