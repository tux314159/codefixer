use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub enum ProblemLanguage {
    Unknown = 0,
    Cpp = 1,
    C = 2,
    Python = 3,
}

impl From<i64> for ProblemLanguage {
    fn from(t: i64) -> Self {
        use ProblemLanguage::*;
        match t {
            1 => Cpp,
            2 => C,
            3 => Python,
            _ => Unknown,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub enum ProblemType {
    Unknown = 0,
    Batch = 1,
    Interactive = 2,
    Communication = 3,
}

impl From<i64> for ProblemType {
    fn from(t: i64) -> Self {
        use ProblemType::*;
        match t {
            1 => Batch,
            2 => Interactive,
            3 => Communication,
            _ => Unknown,
        }
    }
}
