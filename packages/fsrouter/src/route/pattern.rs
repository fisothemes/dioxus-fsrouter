use crate::ParseError;
use indexmap::IndexSet as Set;
use std::collections::HashMap;

pub type RoutePriority = usize;

/// A segment in a route pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Segment {
    /// A static segment like "user" or "posts"
    Static(String),
    /// A parameter segment like ":id" or ":slug"
    Param(String),
    /// A catch-all segment like ":..segments" (matches rest of path)
    CatchAll(String),
}

/// A parsed route pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePattern {
    /// The original path string (e.g. "/user/:id")
    raw: String,
    /// The parsed segments
    segments: Vec<Segment>,
    /// Priority for route matching (higher = match first)
    priority: RoutePriority,
}

impl RoutePattern {
    /// Parse a route path into a pattern
    ///
    /// # Example
    /// ```
    ///  use dioxus_fsrouter::route::pattern::RoutePattern;
    ///
    ///  let pattern = RoutePattern::parse("/user/:id");
    ///  assert_eq!(pattern.segments().len(), 2);
    ///
    pub fn parse(path: &str) -> Result<Self, ParseError> {
        if path.is_empty() {
            return Err(ParseError::EmptyPath);
        }

        if !path.starts_with('/') {
            return Err(ParseError::MissingLeadingSlash {
                route: path.to_string(),
            });
        }

        if path.starts_with('/') && path.ends_with('/') {
            return Err(ParseError::ContainsTrailingSlash {
                route: path.to_string(),
            });
        }

        if path.contains("//") {
            return Err(ParseError::DoubleSlash {
                route: path.to_string(),
            });
        }

        let mut segments = Vec::new();

        let mut contains_catch_all = false;

        for segment in path.split('/') {
            if contains_catch_all {
                return Err(ParseError::CatchAllNotLastParam {
                    route: path.to_string(),
                });
            }

            if segment.contains(' ') {
                return Err(ParseError::ContainsWhitespace {
                    route: path.to_string(),
                });
            }

            if segment.contains('*') {
                return Err(ParseError::WildcardsNotSupported {
                    route: path.to_string(),
                });
            }

            if segment.starts_with(':') && segment.ends_with(':') {
                return Err(ParseError::EmptyParam {
                    route: path.to_string(),
                });
            }

            if !segment.starts_with(':') && segment.contains(':') {
                return Err(ParseError::InvalidParam {
                    param: segment.to_string(),
                    route: path.to_string(),
                    reason: "ambiguous colon usage (e.g. '/user/id:action')".to_string(),
                });
            }

            if let Some(rest) = segment.strip_prefix(":..") {
                segments.push(Segment::CatchAll(rest.to_string()));
                contains_catch_all = true;
            } else if let Some(param) = segment.strip_prefix(':') {
                segments.push(Segment::Param(param.to_string()))
            } else {
                segments.push(Segment::Static(segment.to_string()))
            }
        }

        Ok(Self {
            raw: path.to_string(),
            priority: calculate_priority(&segments),
            segments,
        })
    }

    /// Check if this pattern matches the given URL path
    ///
    /// The URL is normalised before matching (removes query strings, etc.)
    /// Returns Some(params) if it matches, None otherwise.
    /// Parameter values are URL-decoded.
    pub fn matches(&self, url: &str) -> Option<HashMap<String, String>> {
        // TODO: make Map<String, enum>, where vec<string> or string variant, enum is called ParamType
        // Normalise the URL first
        let normalized = normalize_url(url);

        let url_segments: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

        let has_catch_all = self
            .segments
            .last()
            .map_or(false, |s| matches!(s, Segment::CatchAll(_)));

        // Must match segments before the catch-all
        if has_catch_all && url_segments.len() < self.segments.len().saturating_sub(1) {
            return None;
        // Must have the same number of segments
        } else if !has_catch_all && url_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, url_seg) in self.segments.iter().zip(url_segments.iter()) {
            match pattern_seg {
                Segment::Static(expected) => {
                    // Static segments must match exactly
                    if expected != url_seg {
                        return None;
                    }
                }
                Segment::Param(name) => {
                    // Decode the parameter value
                    let decoded = decode_url_segment(url_seg)?;
                    params.insert(name.clone(), decoded);
                }
                Segment::CatchAll(name) => {
                    let start_index = self.segments.len().saturating_sub(1);

                    let remaining_segments = &url_segments[start_index..];

                    let decoded = remaining_segments
                        .iter()
                        .map(|seg| decode_url_segment(seg))
                        .collect::<Option<Vec<_>>>()?
                        .join("/");

                    params.insert(name.clone(), decoded);
                }
            }
        }

        Some(params)
    }

    /// The original path string (e.g. "/user/:id")
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// The parsed segments
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Priority for route matching (higher = match first)
    pub fn priority(&self) -> RoutePriority {
        self.priority
    }

    /// Get all parameter names in this pattern
    pub fn param_names(&self) -> Vec<&str> {
        self.segments
            .iter()
            .filter_map(|seg| match seg {
                Segment::Param(name) => Some(name.as_str()),
                Segment::CatchAll(name) => Some(name.as_str()),
                Segment::Static(_) => None,
            })
            .collect()
    }

    /// Check if this pattern has any parameters
    pub fn has_params(&self) -> bool {
        self.segments
            .iter()
            .any(|seg| matches!(seg, Segment::Param(_) | Segment::CatchAll(_)))
    }

    /// Check if this pattern is static (no parameters)
    pub fn is_static(&self) -> bool {
        !self.has_params()
    }
}

/// Calculate priority for this pattern using position-weighted scoring
///
/// Rules:
/// - Static segments: 10,000 points (base)
/// - Dynamic segments: 1,000 points (base)
/// - Catch-all segments: 100 points (base)
/// - Position multiplier: Earlier segments have more weight
/// - Length bonus: +1 per segment (tiebreaker)
///
/// Formula:
/// ```text
/// for segment at position i (0-indexed):
///     position_weight = (total_segments - i)
///     if Static: score += 10,000 × position_weight
///     if Dynamic: score += 1,000 × position_weight
///     if Catch-all: score += 100 × position_weight
/// score += total_segments
/// ```
///
/// Examples:
/// ```text
/// /user/new      = 10,000×2 + 10,000×1 + 2 = 30,002
/// /user/:id      = 10,000×2 + 1,000×1 + 2  = 21,002
/// /user/:..rest  = 10,000×2 + 100×1 + 2    = 20,102
/// /:user/:id     = 1,000×2 + 1,000×1 + 2   = 3,002
/// ```
pub fn calculate_priority(segments: &[Segment]) -> RoutePriority {
    let mut priority = 0;
    let len = segments.len();

    for (index, segment) in segments.iter().enumerate() {
        let position_weight = len - index;

        let base_score = match segment {
            Segment::Static(_) => 10_000,
            Segment::Param(_) => 1_000,
            Segment::CatchAll(_) => 100,
        };

        priority += base_score * position_weight;
    }

    // Length bonus for tiebreaking
    priority += len;

    priority
}

/// Normalise a URL path for consistent matching
///
/// This function:
/// - Removes query strings and fragments
/// - Ensures leading slash
/// - Removes trailing slash (except root)
/// - Collapses multiple slashes
///
/// # Examples
/// ```
///  use dioxus_fsrouter::route::pattern::normalize_url;
///
///  assert_eq!(normalize_url("/about"), "/about");
///  assert_eq!(normalize_url("/about/"), "/about");
///  assert_eq!(normalize_url("/about?query=1"), "/about");
///  assert_eq!(normalize_url("//about//"), "/about");
/// ```
pub fn normalize_url(url: &str) -> String {
    // 1. Remove query string and fragment
    let url = url
        .split('?')
        .next()
        .and_then(|p| p.split('#').next())
        .unwrap_or(url);

    // 2. Split into segments, filtering empty ones (handles // and trailing /)
    let segments: Vec<&str> = url.split('/').filter(|s| !s.is_empty()).collect();

    // 3. Rebuild path
    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}

/// Decode a URL segment (parameter value)
///
/// Returns None if the segment contains invalid URL encoding.
///
/// # Examples
/// ```
///  use dioxus_fsrouter::route::pattern::decode_url_segment;
///
///  assert_eq!(decode_url_segment("hello"), Some("hello".to_string()));
///  assert_eq!(decode_url_segment("hello%20world"), Some("hello world".to_string()));
///  assert_eq!(decode_url_segment("100%25"), Some("100%".to_string()));
/// ```
pub fn decode_url_segment(segment: &str) -> Option<String> {
    urlencoding::decode(segment).ok().map(|s| s.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Pattern Parsing Tests =====

    #[test]
    fn test_parse_static_route() {
        let pattern = RoutePattern::parse("/about").unwrap();
        assert_eq!(pattern.segments.len(), 1);
        assert_eq!(pattern.segments[0], Segment::Static("about".to_string()));
        assert!(pattern.is_static());
    }

    #[test]
    fn test_parse_single_param() {
        let pattern = RoutePattern::parse("/user/:id").unwrap();
        assert_eq!(pattern.segments.len(), 2);
        assert_eq!(pattern.segments[0], Segment::Static("user".to_string()));
        assert_eq!(pattern.segments[1], Segment::Param("id".to_string()));
        assert!(pattern.has_params());
    }

    #[test]
    fn test_parse_multiple_params() {
        let pattern = RoutePattern::parse("/user/:user_id/post/:post_id").unwrap();
        assert_eq!(pattern.segments.len(), 4);
        assert_eq!(pattern.param_names(), vec!["user_id", "post_id"]);
    }

    // ===== Pattern Matching Tests =====

    #[test]
    fn test_match_static_route() {
        let pattern = RoutePattern::parse("/about").unwrap();
        assert!(pattern.matches("/about").is_some());
        assert!(pattern.matches("/contact").is_none());
        assert!(pattern.matches("/about/more").is_none());
    }

    #[test]
    fn test_match_with_params() {
        let pattern = RoutePattern::parse("/user/:id").unwrap();

        let params = pattern.matches("/user/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        assert!(pattern.matches("/user").is_none());
        assert!(pattern.matches("/user/123/extra").is_none());
    }

    #[test]
    fn test_match_multiple_params() {
        let pattern = RoutePattern::parse("/user/:user_id/post/:post_id").unwrap();

        let params = pattern.matches("/user/alice/post/42").unwrap();
        assert_eq!(params.get("user_id"), Some(&"alice".to_string()));
        assert_eq!(params.get("post_id"), Some(&"42".to_string()));
    }

    #[test]
    fn test_match_with_trailing_slash() {
        let pattern = RoutePattern::parse("/about").unwrap();
        // Trailing slashes are normalised away
        assert!(pattern.matches("/about/").is_some());
    }

    #[test]
    fn test_match_with_query_string() {
        let pattern = RoutePattern::parse("/search").unwrap();
        // Query strings are removed during normalisation
        assert!(pattern.matches("/search?q=rust").is_some());
    }

    // ===== URL Decoding Tests =====

    #[test]
    fn test_decode_simple_param() {
        let pattern = RoutePattern::parse("/user/:name").unwrap();
        let params = pattern.matches("/user/john").unwrap();
        assert_eq!(params.get("name"), Some(&"john".to_string()));
    }

    #[test]
    fn test_decode_url_encoded_param() {
        let pattern = RoutePattern::parse("/user/:name").unwrap();
        let params = pattern.matches("/user/john%20doe").unwrap();
        assert_eq!(params.get("name"), Some(&"john doe".to_string()));
    }

    #[test]
    fn test_decode_special_chars() {
        let pattern = RoutePattern::parse("/search/:query").unwrap();
        let params = pattern.matches("/search/rust%26web").unwrap();
        assert_eq!(params.get("query"), Some(&"rust&web".to_string()));
    }

    #[test]
    fn test_decode_percent_sign() {
        let pattern = RoutePattern::parse("/discount/:amount").unwrap();
        let params = pattern.matches("/discount/50%25").unwrap();
        assert_eq!(params.get("amount"), Some(&"50%".to_string()));
    }

    // ===== Priority Tests =====

    #[test]
    fn test_priority_calculation() {
        let static_route = RoutePattern::parse("/about").unwrap();
        let dynamic_route = RoutePattern::parse("/user/:id").unwrap();
        let mixed_route = RoutePattern::parse("/user/posts/:id").unwrap();

        // /about: 10,000×1 + 1 = 10,001
        assert_eq!(static_route.priority, 10_001);

        // /user/:id: 10,000×2 + 1,000×1 + 2 = 21,002
        assert_eq!(dynamic_route.priority, 21_002);

        // /user/posts/:id: 10,000×3 + 10,000×2 + 1,000×1 + 3 = 51,003
        assert_eq!(mixed_route.priority, 51_003);
    }

    #[test]
    fn test_priority_ordering() {
        let routes = vec![
            RoutePattern::parse("/user/:id").unwrap(),       // 21,002
            RoutePattern::parse("/user/new").unwrap(),       // 30,002
            RoutePattern::parse("/user/:id/edit").unwrap(),  // 32,003
            RoutePattern::parse("/user/new/posts").unwrap(), // 40,003
        ];

        let mut sorted = routes.clone();
        sorted.sort_by(|a, b| b.priority().cmp(&a.priority()));

        // Check order (the highest priority first)
        assert_eq!(sorted[0].raw, "/user/new/posts"); // 40,003
        assert_eq!(sorted[1].raw, "/user/:id/edit"); // 32,003
        assert_eq!(sorted[2].raw, "/user/new"); // 30,002
        assert_eq!(sorted[3].raw, "/user/:id"); // 21,002
    }

    #[test]
    fn test_static_beats_dynamic_at_same_position() {
        let static_route = RoutePattern::parse("/user/new").unwrap();
        let dynamic_route = RoutePattern::parse("/user/:id").unwrap();

        // Both have 2 segments, but static should win
        assert!(static_route.priority() > dynamic_route.priority());
    }

    // ===== URL Normalization Tests =====

    #[test]
    fn test_normalize_url_basic() {
        assert_eq!(normalize_url("/about"), "/about");
        assert_eq!(normalize_url("/"), "/");
        assert_eq!(normalize_url(""), "/");
    }

    #[test]
    fn test_normalize_url_trailing_slash() {
        assert_eq!(normalize_url("/about/"), "/about");
        assert_eq!(normalize_url("/user/profile/"), "/user/profile");
    }

    #[test]
    fn test_normalize_url_no_leading_slash() {
        assert_eq!(normalize_url("about"), "/about");
        assert_eq!(normalize_url("user/profile"), "/user/profile");
    }

    #[test]
    fn test_normalize_url_multiple_slashes() {
        assert_eq!(normalize_url("//about//"), "/about");
        assert_eq!(normalize_url("/user//profile"), "/user/profile");
        assert_eq!(normalize_url("///"), "/");
    }

    #[test]
    fn test_normalize_url_query_string() {
        assert_eq!(normalize_url("/search?q=rust"), "/search");
        assert_eq!(normalize_url("/search?q=rust&page=2"), "/search");
    }

    #[test]
    fn test_normalize_url_fragment() {
        assert_eq!(normalize_url("/docs#introduction"), "/docs");
        assert_eq!(normalize_url("/about#team"), "/about");
    }

    #[test]
    fn test_normalize_url_combined() {
        assert_eq!(normalize_url("/search?q=rust#results"), "/search");
        assert_eq!(normalize_url("//api//v1/?format=json#response"), "/api/v1");
    }

    #[test]
    fn test_root_route() {
        let pattern = RoutePattern::parse("/").unwrap();
        assert_eq!(pattern.segments.len(), 0);
        assert!(pattern.matches("/").is_some());
        assert!(pattern.matches("/about").is_none());
    }
}
