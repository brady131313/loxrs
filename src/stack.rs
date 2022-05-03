#[derive(Debug)]
pub struct Stack<T, const N: usize> {
    data: [T; N],
    top: usize
}

impl<T: Copy + Default, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        Self {
            data: [T::default(); N],
            top: 0
        }
    }

    pub fn push(&mut self, value: T) {
        self.data[self.top] = value;
        self.top += 1
    }

    pub fn pop(&mut self) -> &T {
        self.top -= 1;
        &self.data[self.top]
    }

    pub fn reset(&mut self) {
        self.top = 0
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a Stack<T, N> {
    type Item = &'a T;
    type IntoIter = StackIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        StackIterator {
            stack: self,
            index: 0
        }
    }
}

pub struct StackIterator<'a, T, const N: usize> {
    stack: &'a Stack<T, N>,
    index: usize
}

impl<'a, T, const N: usize> Iterator for StackIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.stack.top {
            let value = &self.stack.data[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack() {
        let mut stack: Stack<_, 5> = Stack::new();
        println!("{stack:?}");
        stack.push(5);
        stack.push(5);
        stack.push(5);
        stack.push(5);
        stack.push(5);
        println!("{stack:?}");
        stack.pop();
        println!("{stack:?}");
        panic!()
    }
}
