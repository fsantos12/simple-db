#[derive(Debug, Clone, Default)]
pub struct GroupDefinition(Vec<String>);

impl GroupDefinition {
    // Contructors
    pub fn new(group: Vec<String>) -> Self {
        Self(group)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    // Vec like methods
    pub fn push(mut self, field: String) -> Self {
        self.0.push(field);
        self
    }

    pub fn append(mut self, other: &mut GroupDefinition) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    // --- Basic ---
    pub fn field<F: Into<String>>(self, field: F) -> Self {
        self.push(field.into())
    }
}


impl From<Vec<String>> for GroupDefinition {
    fn from(v: Vec<String>) -> Self { Self(v) }
}

impl From<GroupDefinition> for Vec<String> {
    fn from(d: GroupDefinition) -> Self { d.0 }
}

impl IntoIterator for GroupDefinition {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a GroupDefinition {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<String> for GroupDefinition {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}