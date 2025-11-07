use std::collections::HashMap;

/// Route parameters extracted from the current path
#[derive(Clone)]
pub struct RouteParams(pub HashMap<String, String>);

/// Extract parameters from a path based on a pattern
///
/// # Example
/// ```
/// use dioxus_fsrouter::params::extract_params;
///
/// let params = extract_params("/blog/123", "/blog/:id");
/// assert_eq!(params.get("id"), Some(&"123".to_string()));
/// ```
pub fn extract_params(path: &str, pattern: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    let path_segments: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let pattern_segments: Vec<&str> = pattern
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    if path_segments.len() != pattern_segments.len() {
        return params;
    }

    for (idx, pattern_seg) in pattern_segments.iter().enumerate() {
        if pattern_seg.starts_with(':') {
            let param_name = &pattern_seg[1..];
            params.insert(param_name.to_string(), path_segments[idx].to_string());
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_param() {
        let params = extract_params("/blog/123", "/blog/:id");
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_extract_multiple_params() {
        let params = extract_params("/user/john/post/456", "/user/:username/post/:id");
        assert_eq!(params.get("username"), Some(&"john".to_string()));
        assert_eq!(params.get("id"), Some(&"456".to_string()));
    }

    #[test]
    fn test_no_match_different_length() {
        let params = extract_params("/blog", "/blog/:id");
        assert!(params.is_empty());
    }
}