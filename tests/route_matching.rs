#[cfg(test)]
mod tests {
    use dioxus_fsrouter::params::extract_params;

    #[test]
    fn test_exact_match() {
        let params = extract_params("/about", "/about");
        assert!(params.is_empty());
    }

    #[test]
    fn test_single_param() {
        let params = extract_params("/user/123", "/user/:id");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_multiple_params() {
        let params = extract_params("/user/john/post/456", "/user/:username/post/:id");
        assert_eq!(params.get("username"), Some(&"john".to_string()));
        assert_eq!(params.get("id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_trailing_slash() {
        let params = extract_params("/user/123/", "/user/:id");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_no_match() {
        let params = extract_params("/user/123/extra", "/user/:id");
        assert!(params.is_empty());
    }
}