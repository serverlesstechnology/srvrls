use validator::{Validate, ValidationError};

#[cfg(test)]
mod validation_tests {
    use super::*;


    #[derive(Default, Debug, Validate)]
    struct TestDto {
        #[validate(length(equal = 8, message="requires 8 chars"))]
        id: String,
        #[validate(length(min = 1), custom = "validate_no_equis")]
        name: String,
    }
    fn validate_no_equis(name: &str) -> Result<(), ValidationError> {
        if name.to_lowercase().contains("x") {
            return Err(ValidationError::new("no x's allowed"));
        }
        Ok(())
    }

    #[test]
    fn test_ok() {
        let dto = TestDto {
            id: "anything".to_string(),
            name: "this is fine".to_string()
        };
        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_short_id() {
        let dto = TestDto {
            id: "short".to_string(),
            name: "this is fine".to_string()
        };
        let errs = dto.validate().err().unwrap();
        assert_eq!(errs.errors().keys().len(), 1);
        let err_map = errs.field_errors();
        let field_errors = err_map.get("id").unwrap();
        assert_eq!(field_errors.len(), 1);
        assert_eq!(field_errors[0].code.as_ref(), "length");
        assert_eq!(field_errors[0].message.as_ref().unwrap(), "requires 8 chars");
    }

    #[test]
    fn test_bad_name() {
        let dto = TestDto {
            id: "anything".to_string(),
            name: "this has x's".to_string()
        };
        let errs = dto.validate().err().unwrap();
        assert_eq!(errs.errors().keys().len(), 1);
        let err_map = errs.field_errors();
        let field_errors = err_map.get("name").unwrap();
        assert_eq!(field_errors.len(), 1);
        print!("{:?}\n", field_errors[0]);
        assert_eq!(field_errors[0].code.as_ref(), "no x's allowed");
    }
}