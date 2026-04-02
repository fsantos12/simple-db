use crate::query::projections::projection::Projection;

#[derive(Debug, Clone, Default)]
pub struct ProjectionDefinition(Vec<Projection>);

impl ProjectionDefinition {
    // Contructors
    pub fn new(projections: Vec<Projection>) -> Self {
        Self(projections)
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    // Vec like methods
    pub fn push(mut self, projection: Projection) -> Self {
        self.0.push(projection);
        self
    }

    pub fn append(mut self, other: &mut ProjectionDefinition) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn pop(&mut self) -> Option<Projection> {
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
        self.push(Projection::Field(field.into()))
    }

    pub fn alias<F: Into<String>, A: Into<String>>(self, field: F, alias: A) -> Self {
        self.push(Projection::Alias(field.into(), alias.into()))
    }

    // --- Aggregations ---
    pub fn count_all(self) -> Self {
        self.push(Projection::CountAll)
    }

    pub fn count<F: Into<String>>(self, field: F) -> Self {
        self.push(Projection::Count(field.into()))
    }

    pub fn sum<F: Into<String>>(self, field: F) -> Self {
        self.push(Projection::Sum(field.into()))
    }

    pub fn avg<F: Into<String>>(self, field: F) -> Self {
        self.push(Projection::Avg(field.into()))
    }

    pub fn min<F: Into<String>>(self, field: F) -> Self {
        self.push(Projection::Min(field.into()))
    }

    pub fn max<F: Into<String>>(self, field: F) -> Self {
        self.push(Projection::Max(field.into()))
    }
}


impl From<Vec<Projection>> for ProjectionDefinition {
    fn from(v: Vec<Projection>) -> Self { Self(v) }
}

impl From<ProjectionDefinition> for Vec<Projection> {
    fn from(d: ProjectionDefinition) -> Self { d.0 }
}

impl IntoIterator for ProjectionDefinition {
    type Item = Projection;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ProjectionDefinition {
    type Item = &'a Projection;
    type IntoIter = std::slice::Iter<'a, Projection>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<Projection> for ProjectionDefinition {
    fn extend<T: IntoIterator<Item = Projection>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}