use std::collections::HashMap;

/// Interned string type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IString(usize);

#[derive(Debug)]
pub struct StringInterner {
    map: HashMap<String, IString>,
    vals: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            vals: Vec::new(),
        }
    }

    pub fn intern<S: Into<String>>(&mut self, str: S) -> IString {
        let str = str.into();
        if let Some(val) = self.map.get(&str) {
            *val
        } else {
            let istr = IString(self.vals.len());
            self.vals.push(str.clone());
            self.map.insert(str, istr);
            istr
        }
    }

    pub fn get(&self, istr: IString) -> &str {
        &self.vals[istr.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern() {
        let mut interner = StringInterner::new();
        let a = interner.intern("this is a test");
        assert_eq!(interner.get(a), "this is a test");
    }
}
