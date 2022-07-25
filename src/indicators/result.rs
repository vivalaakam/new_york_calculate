#[derive(Debug)]
pub enum IndicatorsError {
    InvalidOption(String),
}

pub type IndicatorsResult<T> = std::result::Result<T, IndicatorsError>;
