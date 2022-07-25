use crate::hash_md5::hash_md5;

pub fn get_applicant_id<T>(interval: T, start: T, end: T, model_id: T) -> String
where
    T: Into<String>,
{
    hash_md5(format!(
        "{}:{}:{}:{}",
        interval.into(),
        start.into(),
        end.into(),
        model_id.into()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_applicant_id() {
        assert_eq!(
            "3d66ff22fd43e3b37d3a4a06322cc636",
            get_applicant_id(1.to_string(), 2.to_string(), 3.to_string(), 4.to_string())
        );
    }
}
