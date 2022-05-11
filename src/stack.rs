const INITIAL_STACK_SIZE: usize = u8::MAX as usize;

#[derive(Debug)]
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::with_capacity(INITIAL_STACK_SIZE),
        }
    }

    pub fn push<V: Into<T>>(&mut self, value: V) {
        self.data.push(value.into())
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn peek(&self, distance: usize) -> Option<&T> {
        let idx = self.data.len().checked_sub(distance + 1)?;
        self.data.get(idx)
    }

    pub fn reset(&mut self) {
        self.data.clear()
    }
}

impl<'a, T> IntoIterator for &'a Stack<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let mut stack: Stack<i32> = Stack::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.pop().unwrap(), 2);
        assert_eq!(stack.pop().unwrap(), 1);

        assert!(stack.peek(0).is_none());
        stack.push(5);
        stack.push(3);
        assert_eq!(stack.peek(0).unwrap(), &3);
        assert_eq!(stack.peek(1).unwrap(), &5);
    }
}
