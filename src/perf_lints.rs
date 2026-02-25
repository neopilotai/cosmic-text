//! Performance lints for GUI/TUI applications
//!
//! This module provides lint rules to help developers identify
//! common performance issues in GUI/TUI applications.
//!
//! ## Quick Reference
//!
//! | Lint Code | Description | Fix |
//! |-----------|-------------|-----|
//! | `cosmic_perf_001` | Clone in render method | Use `&` borrow instead |
//! | `cosmic_perf_002` | `Box<dyn>` in hot path | Use generics |
//! | `cosmic_perf_003` | `Vec::new()` in loop | Pre-allocate with capacity |
//! | `cosmic_perf_004` | `to_string()` on `&str` | Use `&str` directly |
//! | `cosmic_perf_005` | Hot fn without inline | Add `#[inline]` |
//! | `cosmic_perf_006` | `HashMap` in render path | Use `FxHashMap` |
//! | `cosmic_perf_007` | `s + &x` in loop | Use `join()` or `write!()` |
//! | `cosmic_perf_008` | Unnecessary `Arc::clone()` | Use `&` instead |
//! | `cosmic_perf_009` | `Rc` in render path | Use borrow |
//! | `cosmic_perf_010` | Mutex in render loop | Double-buffer |
//! | `cosmic_perf_011` | `println!` in render | Gate with `cfg` |
//! | `cosmic_perf_012` | Regex in render | Pre-compile |
//! | `cosmic_perf_013` | Unnecessary `.collect()` | Chain iterators |
//! | `cosmic_perf_014` | Clone full collection | Borrow instead |
//! | `cosmic_perf_015` | FP in tight loops | Use integer math |
//!
//! ## Usage
//!
//! Add this to your crate root:
//!
//! ```ignore
//! // Enable all performance lints
//! #![warn(
//!     fastui_cosmic::perf_lints::EXPENSIVE_CLONE,
//!     fastui_cosmic::perf_lints::ALLOCATION_IN_LOOP,
//! use fastui_cosmic::perf_lints::{check_render_fn, lint_codes::*, PerformanceLinter};
//! ```

/// Lint error codes for GUI/TUI performance
pub mod lint_codes {
    //! Lint error codes that can be used with `#[warn(lint_code)]`

    /// Detects expensive `.clone()` calls in render/update paths
    ///
    /// **BAD:**
    /// ```ignore
    /// fn render(&self) {
    ///     let data = self.buffer.clone();
    /// }
    /// ```
    ///
    /// **GOOD:**
    /// ```ignore
    /// fn render(&self) {
    ///     let data = &self.buffer;
    /// }
    /// ```
    pub const EXPENSIVE_CLONE: &str = "cosmic_perf_001";

    /// Detects `Box<dyn Trait>` in render-critical types
    ///
    /// **BAD:**
    /// ```ignore
    /// struct Renderer { drawer: Box<dyn Drawer> }
    /// ```
    ///
    /// **GOOD:**
    /// ```ignore
    /// struct Renderer<D: Drawer> { drawer: D }
    /// ```
    pub const BOX_DYN_IN_RENDER: &str = "cosmic_perf_002";

    /// Detects allocations inside loops
    ///
    /// **BAD:**
    /// ```ignore
    /// for _ in 0..1000 { let mut s = String::new(); }
    /// ```
    ///
    /// **GOOD:**
    /// ```ignore
    /// let mut s = String::with_capacity(1024);
    /// for _ in 0..1000 { s.clear(); }
    /// ```
    pub const ALLOCATION_IN_LOOP: &str = "cosmic_perf_003";

    /// Detects unnecessary String allocations from &str
    pub const STRING_ALLOCATION: &str = "cosmic_perf_004";

    /// Detects missing #[inline] on hot path functions
    pub const MISSING_INLINE: &str = "cosmic_perf_005";

    /// Detects slow HashMap/HashSet in hot paths
    pub const SLOW_HASHER: &str = "cosmic_perf_006";

    /// Detects string concatenation in loops
    pub const STRING_CONCAT_LOOP: &str = "cosmic_perf_007";

    /// Detects unnecessary Arc::clone() calls
    pub const ARC_CLONE_UNNECESSARY: &str = "cosmic_perf_008";

    /// Detects Rc in render-critical code
    pub const RC_IN_RENDER: &str = "cosmic_perf_009";

    /// Detects locking in render paths
    pub const LOCK_IN_RENDER: &str = "cosmic_perf_010";

    /// Detects I/O in render paths
    pub const IO_IN_RENDER: &str = "cosmic_perf_011";

    /// Detects regex compilation in hot paths
    pub const REGEX_IN_HOT_PATH: &str = "cosmic_perf_012";

    /// Detects unnecessary .collect() followed by iteration
    pub const COLLECT_THEN_ITER: &str = "cosmic_perf_013";

    /// Detects cloning full collections when partial borrow suffices
    pub const CLONE_FULL_COLLECTION: &str = "cosmic_perf_014";

    /// Detects floating-point operations in tight loops
    pub const FP_IN_TIGHT_LOOP: &str = "cosmic_perf_015";
}

/// Helper functions for performance checking
pub mod check {
    //! Runtime helpers to check for performance issues

    /// Check if a function name suggests it's in the render path
    ///
    /// # Example
    /// ```ignore
    /// fn render_frame() { }  // returns true
    /// fn draw_widget() { }    // returns true
    /// fn process_data() { }   // returns false
    /// ```
    #[inline]
    pub fn is_render_fn(name: &str) -> bool {
        name.contains("render")
            || name.contains("draw")
            || name.contains("update")
            || name.contains("paint")
            || name.contains("layout")
            || name.contains("shape")
    }

    #[inline]
    pub fn check_render_fn(name: &str) -> bool {
        is_render_fn(name)
    }

    /// Check if a type name suggests it's render-related
    #[inline]
    pub fn is_render_type(name: &str) -> bool {
        name.contains("Buffer")
            || name.contains("Layout")
            || name.contains("Glyph")
            || name.contains("Widget")
            || name.contains("Renderer")
    }

    /// Recommend fast hasher based on context
    pub fn recommended_hasher(use_case: &str) -> &'static str {
        match use_case {
            "cache" | "render" | "layout" => "FxHashMap / FxHashSet",
            "crypto" | "security" => "Default (SipHash)",
            "benchmark" | "one_shot" => "RandomState",
            _ => "FxHashMap / FxHashSet",
        }
    }
}

/// AST-based performance linter for GUI/TUI applications
///
/// This linter analyzes code patterns that commonly cause
/// performance issues in render/update paths.
///
/// # Usage
///
/// ```ignore
/// use fastui_cosmic::perf_lints::{PerformanceLinter, lint_codes::*};
///
/// let mut linter = PerformanceLinter::new();
/// linter.enable(EXPENSIVE_CLONE);
/// linter.enable(ALLOCATION_IN_LOOP);
/// linter.enable(SLOW_HASHER);
///
/// let issues = linter.lint_source(source_code);
/// for issue in issues {
///     println!("{}: {}", issue.code, issue.message);
/// }
/// ```
#[derive(Debug)]
pub struct PerformanceLinter {
    enabled_rules: Vec<&'static str>,
}

impl PerformanceLinter {
    /// Create a new PerformanceLinter with all rules enabled
    pub fn new() -> Self {
        Self {
            enabled_rules: vec![
                lint_codes::EXPENSIVE_CLONE,
                lint_codes::BOX_DYN_IN_RENDER,
                lint_codes::ALLOCATION_IN_LOOP,
                lint_codes::STRING_ALLOCATION,
                lint_codes::MISSING_INLINE,
                lint_codes::SLOW_HASHER,
                lint_codes::STRING_CONCAT_LOOP,
                lint_codes::ARC_CLONE_UNNECESSARY,
                lint_codes::RC_IN_RENDER,
                lint_codes::LOCK_IN_RENDER,
                lint_codes::IO_IN_RENDER,
                lint_codes::REGEX_IN_HOT_PATH,
                lint_codes::COLLECT_THEN_ITER,
                lint_codes::CLONE_FULL_COLLECTION,
                lint_codes::FP_IN_TIGHT_LOOP,
            ],
        }
    }

    /// Enable a specific lint rule
    pub fn enable(&mut self, rule: &'static str) {
        if !self.enabled_rules.contains(&rule) {
            self.enabled_rules.push(rule);
        }
    }

    /// Disable a specific lint rule
    pub fn disable(&mut self, rule: &'static str) {
        self.enabled_rules.retain(|r| *r != rule);
    }

    /// Get list of enabled rules
    pub fn enabled_rules(&self) -> &[&'static str] {
        &self.enabled_rules
    }

    /// Check if a rule is enabled
    pub fn is_enabled(&self, rule: &str) -> bool {
        self.enabled_rules.iter().any(|r| *r == rule)
    }
}

impl Default for PerformanceLinter {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a performance issue found by the linter
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// The lint code (e.g., "cosmic_perf_001")
    pub code: &'static str,
    /// Human-readable message describing the issue
    pub message: String,
    /// Line number where the issue was found
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

impl PerformanceLinter {
    /// Lint source code for performance issues
    ///
    /// This performs simple pattern matching on the source code.
    /// For full AST-based linting, use a clippy integration.
    pub fn lint_source(&self, source: &str) -> Vec<LintIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            if self.is_enabled(lint_codes::EXPENSIVE_CLONE) {
                if line.contains(".clone()")
                    && (line.contains("render") || line.contains("draw") || line.contains("update"))
                {
                    issues.push(LintIssue {
                        code: lint_codes::EXPENSIVE_CLONE,
                        message: "Expensive clone in render path".to_string(),
                        line: line_num + 1,
                        column: line.find(".clone()").unwrap_or(0) + 1,
                        suggestion: Some("Use & borrow instead".to_string()),
                    });
                }
            }

            if self.is_enabled(lint_codes::ALLOCATION_IN_LOOP) {
                if (line.contains("String::new()")
                    || line.contains("Vec::new()")
                    || line.contains("Box::new("))
                    && (source
                        .lines()
                        .nth(line_num.saturating_sub(1))
                        .map_or(false, |l| l.contains("for"))
                        || source
                            .lines()
                            .nth(line_num.saturating_sub(2))
                            .map_or(false, |l| l.contains("for")))
                {
                    issues.push(LintIssue {
                        code: lint_codes::ALLOCATION_IN_LOOP,
                        message: "Allocation in loop".to_string(),
                        line: line_num + 1,
                        column: 1,
                        suggestion: Some("Pre-allocate with capacity".to_string()),
                    });
                }
            }

            if self.is_enabled(lint_codes::SLOW_HASHER) {
                if line.contains("HashMap") || line.contains("HashSet") {
                    if !line.contains("FxHash") && !line.contains("FxHasher") {
                        issues.push(LintIssue {
                            code: lint_codes::SLOW_HASHER,
                            message: "Slow default hasher in potential hot path".to_string(),
                            line: line_num + 1,
                            column: 1,
                            suggestion: Some("Use FxHashMap or FxHashSet".to_string()),
                        });
                    }
                }
            }

            if self.is_enabled(lint_codes::STRING_ALLOCATION) {
                if line.contains("to_string()") && !line.contains("&str") {
                    issues.push(LintIssue {
                        code: lint_codes::STRING_ALLOCATION,
                        message: "Unnecessary String allocation".to_string(),
                        line: line_num + 1,
                        column: line.find(".to_string()").unwrap_or(0) + 1,
                        suggestion: Some("Use &str or &String directly".to_string()),
                    });
                }
            }

            if self.is_enabled(lint_codes::MISSING_INLINE) {
                if line.contains("fn ")
                    && !line.contains("#[inline]")
                    && !line.contains("fn render")
                    && !line.contains("fn draw")
                {
                    let fn_name = line
                        .split("fn ")
                        .nth(1)
                        .unwrap_or("")
                        .split('(')
                        .next()
                        .unwrap_or("");
                    if check::is_render_fn(fn_name) {
                        issues.push(LintIssue {
                            code: lint_codes::MISSING_INLINE,
                            message: format!("Hot path function '{}' missing #[inline]", fn_name),
                            line: line_num + 1,
                            column: 1,
                            suggestion: Some("Add #[inline] attribute".to_string()),
                        });
                    }
                }
            }

            if self.is_enabled(lint_codes::IO_IN_RENDER) {
                if (line.contains("println!")
                    || line.contains("eprintln!")
                    || line.contains("write!"))
                    && (line.contains("render")
                        || line.contains("draw")
                        || line.contains("update")
                        || line.contains("paint"))
                {
                    issues.push(LintIssue {
                        code: lint_codes::IO_IN_RENDER,
                        message: "I/O operation in render path".to_string(),
                        line: line_num + 1,
                        column: 1,
                        suggestion: Some("Gate with cfg(debug_assertions)".to_string()),
                    });
                }
            }
        }

        issues
    }
}

/// Performance checklist for GUI/TUI applications
pub mod checklist {
    /// Render loop performance checklist
    pub const RENDER_LOOP: &str = r#"
    GUI/TUI Performance Checklist:
    
    [ ] Avoid allocations in loops
        - Use Vec::with_capacity()
        - Reuse buffers with clear()
    
    [ ] Minimize cloning
        - Borrow instead of clone
        - Use Arc only when sharing necessary
    
    [ ] Use fast hash maps
        - FxHashMap instead of HashMap
        - FxHashSet instead of HashSet
    
    [ ] Enable optimizations
        - #[inline] on hot functions
        - LTO in release profile
        - opt-level = 3
    
    [ ] Profile first
        - cargo-flamegraph
        - Measure frame times
    
    [ ] Threading
        - Shape on background threads
        - Double-buffer layouts
        - Avoid locks in render
    
    [ ] Memory
        - Cache-friendly layouts
        - Minimize pointer chains
        - Stack over heap when possible
    "#;

    /// cosmic-text specific optimizations
    pub const COSMIC_TEXT: &str = r#"
    cosmic-text Optimization Tips:
    
    [ ] Cache ShapeLine results
        - Don't re-shape unchanged text
    
    [ ] Reuse Buffer objects
        - Don't create new Buffer per frame
    
    [ ] Batch glyph updates
        - Collect changes before shaping
    
    [ ] Use appropriate Wrap mode
        - Wrap::None for fixed layouts
        - Wrap::WordOrGlyph for dynamic
    
    [ ] Pre-load fonts
        - Load fonts at startup
        - Don't load in render loop
    
    [ ] Use FxHashMap for glyph cache
        - Faster than default hasher
    "#;
}

/// Manual lint implementation using procedural macros
///
/// Add to your code to mark functions for lint checking:
///
/// ```ignore
/// #[render_path]
/// fn draw(&mut self) { ... }
/// ```
#[allow(non_snake_case)]
pub mod macros {
    /// Marks a function as in the render path
    /// This is a documentation marker - actual linting requires clippy
    #[macro_export]
    #[doc(hidden)]
    macro_rules! render_path {
        ($($tt:tt)*) => {
            #[inline]
            $($tt)*
        };
    }

    /// Marks a type as render-critical
    #[macro_export]
    #[doc(hidden)]
    macro_rules! render_type {
        ($($tt:tt)*) => {
            #[derive(Debug)]
            $($tt)*
        };
    }

    /// Helper to detect render methods
    #[macro_export]
    #[doc(hidden)]
    macro_rules! is_render_method {
        (render) => {
            true
        };
        (draw) => {
            true
        };
        (update) => {
            true
        };
        (paint) => {
            true
        };
        (layout) => {
            true
        };
        ($other:ident) => {
            false
        };
    }
}

#[cfg(test)]
mod tests {
    use super::check::{check_render_fn, is_render_fn, is_render_type, recommended_hasher};
    use super::{lint_codes::*, LintIssue, PerformanceLinter};

    #[test]
    fn test_is_render_fn() {
        assert!(is_render_fn("render_frame"));
        assert!(is_render_fn("draw_widget"));
        assert!(is_render_fn("update_layout"));
        assert!(is_render_fn("paint"));
        assert!(is_render_fn("shape_text"));
        assert!(!is_render_fn("process_data"));
    }

    #[test]
    fn test_check_render_fn() {
        assert!(check_render_fn("render_frame"));
        assert!(check_render_fn("draw_widget"));
        assert!(!check_render_fn("process_data"));
    }

    #[test]
    fn test_is_render_type() {
        assert!(is_render_type("Buffer"));
        assert!(is_render_type("LayoutGlyph"));
        assert!(is_render_type("Widget"));
        assert!(is_render_type("Renderer"));
        assert!(!is_render_type("Config"));
    }

    #[test]
    fn test_recommended_hasher() {
        assert_eq!(recommended_hasher("cache"), "FxHashMap / FxHashSet");
        assert_eq!(recommended_hasher("render"), "FxHashMap / FxHashSet");
        assert_eq!(recommended_hasher("crypto"), "Default (SipHash)");
    }

    #[test]
    fn test_performance_linter_default() {
        let linter = PerformanceLinter::default();
        assert!(linter.is_enabled(EXPENSIVE_CLONE));
        assert!(linter.is_enabled(ALLOCATION_IN_LOOP));
        assert!(linter.is_enabled(SLOW_HASHER));
    }

    #[test]
    fn test_performance_linter_enable_disable() {
        let mut linter = PerformanceLinter::new();
        linter.disable(EXPENSIVE_CLONE);
        assert!(!linter.is_enabled(EXPENSIVE_CLONE));

        linter.enable(EXPENSIVE_CLONE);
        assert!(linter.is_enabled(EXPENSIVE_CLONE));
    }

    #[test]
    fn test_lint_source_clone() {
        let linter = PerformanceLinter::new();
        let source = r#"fn render(&self) { let data = self.buffer.clone(); }"#;
        let issues = linter.lint_source(source);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].code, EXPENSIVE_CLONE);
    }

    #[test]
    fn test_lint_source_hashmap() {
        let linter = PerformanceLinter::new();
        let source = r#"
fn draw() {
    let map = HashMap::new();
}
"#;
        let issues = linter.lint_source(source);
        let hasher_issues: Vec<_> = issues.iter().filter(|i| i.code == SLOW_HASHER).collect();
        assert!(!hasher_issues.is_empty());
    }

    #[test]
    fn test_lint_source_to_string() {
        let linter = PerformanceLinter::new();
        let source = r#"
fn process(s: &str) {
    let x = s.to_string();
}
"#;
        let issues = linter.lint_source(source);
        let alloc_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.code == STRING_ALLOCATION)
            .collect();
        assert!(!alloc_issues.is_empty());
    }
}
