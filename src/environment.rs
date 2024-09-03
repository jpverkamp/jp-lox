pub trait Environment<T> {
    fn get(&self, key: &str) -> Option<T>;
    fn set(&mut self, key: &str, value: T);
    fn enter(&mut self);
    fn exit(&mut self);
}

pub struct EnvironmentStack<T> {
    stack: Vec<Vec<(String, T)>>,
}

impl<T> EnvironmentStack<T> {
    pub fn new() -> Self {
        Self {
            stack: vec![vec![]],
        }
    }
}

impl<T: Clone> Environment<T> for EnvironmentStack<T> {
    fn get(&self, key: &str) -> Option<T> {
        for frame in self.stack.iter().rev() {
            for (k, v) in frame.iter().rev() {
                if k == key {
                    return Some(v.clone());
                }
            }
        }

        None
    }

    fn set(&mut self, key: &str, value: T) {
        self.stack
            .last_mut()
            .unwrap()
            .push((key.to_string(), value));
    }

    fn enter(&mut self) {
        self.stack.push(vec![]);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }
}
