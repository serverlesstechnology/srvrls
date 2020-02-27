use serde::Serialize;


#[cfg(test)]
mod persistence_tests {
    use super::*;

    #[derive(Default, Debug, Serialize)]
    struct TestDto {
        id: String,
        name: String,
    }

    #[test]
    fn test_ok() {
    }
}