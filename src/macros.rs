/// Match the current path against route patterns
///
/// # Example
/// ```rust,no_run
/// # use dioxus::prelude::*;
/// # use dioxus_fsrouter::*;
/// fn route_resolver(path: String) -> Element {
///     match_route!(&path => {
///         "/" => rsx! { Home {} },
///         "/blog/:id" => rsx! { BlogPost {} },
///         "/about" => rsx! { About {} },
///     })
/// }
/// # fn Home() -> Element { rsx! { div {} } }
/// # fn BlogPost() -> Element { rsx! { div {} } }
/// # fn About() -> Element { rsx! { div {} } }
/// ```
#[macro_export]
macro_rules! match_route {
    ($path:expr => {
        $($pattern:literal => $component:expr),* $(,)?
    }) => {{
        use dioxus::prelude::*;
        
        let path = $path;
        let path_segments: Vec<&str> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        
        $(
            let pattern_segments: Vec<&str> = $pattern
                .trim_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();
            
            if path_segments.len() == pattern_segments.len() {
                let mut params = std::collections::HashMap::new();
                let mut is_match = true;
                
                for (idx, pattern_seg) in pattern_segments.iter().enumerate() {
                    if pattern_seg.starts_with(':') {
                        let param_name = &pattern_seg[1..];
                        params.insert(param_name.to_string(), path_segments[idx].to_string());
                    } else if *pattern_seg != path_segments[idx] {
                        is_match = false;
                        break;
                    }
                }
                
                if is_match {
                    use_context_provider(|| $crate::RouteParams(params));
                    return $component;
                }
            }
        )*
        
        // 404 fallback
        use_context_provider(|| $crate::RouteParams(std::collections::HashMap::new()));
        rsx! {
            div {
                h1 { "404 - Not Found" }
                p { "Route: {path}" }
            }
        }
    }};
}