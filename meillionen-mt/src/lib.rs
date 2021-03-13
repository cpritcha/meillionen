use ndarray::{Array1};
use numpy;

pub mod model;
pub mod extension_columns;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dimension {
    name: String,
    sz: usize
}

impl Dimension {
    pub fn new(name: impl AsRef<str>, sz: usize) -> Self {
        Self {
            name: name.as_ref().to_string(),
            sz
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn size(&self) -> usize {
        self.sz
    }

    pub fn iter(&self) -> impl Iterator<Item=usize> {
        (0..self.size()).into_iter()
    }
}

pub trait IntoPandas: Sized {
    fn into_pandas(self, py: pyo3::Python) -> pyo3::PyResult<&pyo3::types::PyAny>;
}

pub trait FromPandas: Sized {
    fn from_pandas(obj: &pyo3::types::PyAny) -> Result<Self, pyo3::PyErr>;
}

#[derive(Clone, Copy)]
pub enum SliceType {
    Index(usize),
    All
}

impl Default for SliceType {
    fn default() -> Self {
        SliceType::All
    }
}

impl SliceType {
    fn is_index(&self) -> bool {
        if let SliceType::Index(_) = self {
            true
        } else {
            false
        }
    }

    fn is_all(&self) -> bool {
        if let SliceType::All = self {
            true
        } else {
            false
        }
    }
}

pub trait Variable {
    type Elem;
    type Index;

    fn name(&self) -> String;
    fn slice(&self, index: &Self::Index) -> Array1<Self::Elem>;
    fn get_dimensions(&self) -> Vec<Dimension>;
}

pub struct VarView<T> where T: Variable {
    sv: T,
    dim_order: Vec<Dimension>,
    indice_map: Vec<usize>
}

impl<T> VarView<T> where T: Variable {
    pub fn try_new(sv: T, dim_order: Vec<Dimension>) -> Self {
        let mut indice_map = Vec::<usize>::new();
        for dim in sv.get_dimensions().iter() {
            let pos = dim_order.iter().position(|d| d == dim);
            if let Some(ind) = pos {
                indice_map.push(ind);
            }
        }

        Self {
            sv,
            dim_order,
            indice_map
        }
    }
}

impl<T> Variable for VarView<T>
where
    T: Variable<Index=Vec<SliceType>, Elem=T> {
    type Elem = T;
    type Index = Vec<SliceType>;

    fn name(&self) -> String {
        self.sv.name()
    }

    fn slice(&self, index: &Self::Index) -> Array1<Self::Elem> {
        assert_eq!(index.len(), self.indice_map.len());
        let inner_index = self.indice_map.iter()
            .map(|i| index[i.clone()]).collect::<Vec<_>>();
        self.sv.slice(&inner_index)
    }

    fn get_dimensions(&self) -> Vec<Dimension> {
        self.dim_order.clone()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn slicing() {
        let xs = vec!["a", "vc", "qsd"];
        assert_eq!(&["qsd", "a"], &[xs[2], xs[0]])
    }
}