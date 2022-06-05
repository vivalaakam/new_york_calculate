use md5::compute;

pub fn hash_md5<S1>(str: S1) -> String
where
    S1: Into<String>,
{
    format!("{:x}", compute(str.into().as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_md5_from_string() {
        assert_eq!(
            "098f6bcd4621d373cade4e832627b4f6",
            hash_md5("test".to_string())
        );
    }

    #[test]
    fn hash_md5_from_str() {
        assert_eq!("098f6bcd4621d373cade4e832627b4f6", hash_md5("test"));
    }
}
