use serde::{Deserialize, Serialize};





#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MyResult<T, E> {
    Ok(T),
    Err(E),
}