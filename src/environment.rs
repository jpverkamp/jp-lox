use crate::values::Value;

pub trait Environment {
    fn get(&self, key: &str) -> Option<Value>;
    fn set(&mut self, key: &str, value: Value);
    fn enter(&mut self);
    fn exit(&mut self);
}

pub struct EnvironmentStack {
    stack: Vec<Vec<(String, Value)>>,
}

impl EnvironmentStack {
    pub fn new() -> Self {
        Self { stack: vec![vec![]] }
    }
}

impl Environment for EnvironmentStack {
    fn get(&self, key: &str) -> Option<Value> {
        for frame in self.stack.iter().rev() {
            for (k, v) in frame.iter().rev() {
                if k == key {
                    return Some(v.clone());
                }
            }
        }

        None
    }

    fn set(&mut self, key: &str, value: Value) {
        self.stack.last_mut().unwrap().push((key.to_string(), value));
    }

    fn enter(&mut self) {
        self.stack.push(vec![]);
    }

    fn exit(&mut self) {
        self.stack.pop();
    }
}
