use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VecOrSingle<T> {
    Vec(Vec<T>),
    Single(T),
}

impl<T> VecOrSingle<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            VecOrSingle::Vec(vec) => vec,
            VecOrSingle::Single(string) => vec![string],
        }
    }
}
