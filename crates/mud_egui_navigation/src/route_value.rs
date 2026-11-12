//! A typed route parameter value.
//!
//! Route templates such as `/orders/{id:int}` capture path segments and convert
//! them according to the declared constraint. The result is stored as a
//! [`RouteValue`], a small tagged union that keeps the converted type while
//! still allowing lenient coercion at the call site (for example, reading an
//! unconstrained `{id}` segment as an integer via [`RouteValue::as_i64`]).

use std::borrow::Cow;
use std::fmt;

/// A typed value captured from a URL path by a route template.
///
/// `RouteValue` implements [`PartialEq`] but not `Eq`, because it can hold an
/// `f64`.
///
/// # Examples
///
/// ```ignore
/// use crate::route_value::RouteValue;
///
/// let v = RouteValue::Int(42);
/// assert_eq!(v.as_i64(), Some(42));
/// assert_eq!(v.as_f64(), Some(42.0));
/// assert_eq!(v.as_str(), None);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum RouteValue {
    /// An unconstrained segment, a catch-all tail, or a `guid` / `datetime`
    /// segment (kept in textual form).
    Str(String),
    /// A segment constrained with `:int` or `:long`.
    Int(i64),
    /// A segment constrained with `:float`, `:double` or `:decimal`.
    Float(f64),
    /// A segment constrained with `:bool` (`true` / `false`, case-sensitive).
    Bool(bool),
}

impl Default for RouteValue {
    /// Returns an empty [`RouteValue::Str`].
    fn default() -> Self {
        RouteValue::Str(String::new())
    }
}

impl RouteValue {
    /// Returns the inner string slice if (and only if) this is a
    /// [`RouteValue::Str`]. Never allocates.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            RouteValue::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the value as an `i64`.
    ///
    /// [`RouteValue::Int`] is returned directly; [`RouteValue::Str`] is parsed.
    /// `Float` and `Bool` return `None`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            RouteValue::Int(i) => Some(*i),
            RouteValue::Str(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Returns the value as an `f64`.
    ///
    /// `Float` is returned directly, `Int` is widened, `Str` is parsed and
    /// `Bool` returns `None`.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            RouteValue::Float(f) => Some(*f),
            RouteValue::Int(i) => Some(*i as f64),
            RouteValue::Str(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Returns the value as a `bool`.
    ///
    /// `Bool` is returned directly and `Str` is parsed (`"true"` / `"false"`).
    /// Numeric variants return `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RouteValue::Bool(b) => Some(*b),
            RouteValue::Str(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Renders the value as text, borrowing when it is already a string.
    ///
    /// ```ignore
    /// # use crate::route_value::RouteValue;
    /// assert_eq!(RouteValue::from("abc").to_text(), "abc");
    /// assert_eq!(RouteValue::Int(7).to_text(), "7");
    /// ```
    pub fn to_text(&self) -> Cow<'_, str> {
        match self {
            RouteValue::Str(s) => Cow::Borrowed(s.as_str()),
            other => Cow::Owned(other.to_string()),
        }
    }

    /// A short, stable name of the active variant (`"str"`, `"int"`, `"float"`
    /// or `"bool"`), useful for diagnostics.
    pub fn kind(&self) -> &'static str {
        match self {
            RouteValue::Str(_) => "str",
            RouteValue::Int(_) => "int",
            RouteValue::Float(_) => "float",
            RouteValue::Bool(_) => "bool",
        }
    }
}

impl fmt::Display for RouteValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouteValue::Str(s) => f.write_str(s),
            RouteValue::Int(i) => write!(f, "{i}"),
            RouteValue::Float(x) => write!(f, "{x}"),
            RouteValue::Bool(b) => write!(f, "{b}"),
        }
    }
}

impl From<String> for RouteValue {
    fn from(v: String) -> Self {
        RouteValue::Str(v)
    }
}
impl From<&str> for RouteValue {
    fn from(v: &str) -> Self {
        RouteValue::Str(v.to_string())
    }
}
impl From<i64> for RouteValue {
    fn from(v: i64) -> Self {
        RouteValue::Int(v)
    }
}
impl From<i32> for RouteValue {
    fn from(v: i32) -> Self {
        RouteValue::Int(i64::from(v))
    }
}
impl From<f64> for RouteValue {
    fn from(v: f64) -> Self {
        RouteValue::Float(v)
    }
}
impl From<bool> for RouteValue {
    fn from(v: bool) -> Self {
        RouteValue::Bool(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_accessors() {
        let v = RouteValue::Str("hello".into());
        assert_eq!(v.as_str(), Some("hello"));
        assert_eq!(v.as_i64(), None);
        assert_eq!(v.as_f64(), None);
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn str_is_leniently_parsed() {
        assert_eq!(RouteValue::from("12").as_i64(), Some(12));
        assert_eq!(RouteValue::from("-3").as_i64(), Some(-3));
        assert_eq!(RouteValue::from("2.5").as_f64(), Some(2.5));
        assert_eq!(RouteValue::from("true").as_bool(), Some(true));
        assert_eq!(RouteValue::from("false").as_bool(), Some(false));
    }

    #[test]
    fn int_widens_to_float_but_not_str_or_bool() {
        let v = RouteValue::Int(5);
        assert_eq!(v.as_i64(), Some(5));
        assert_eq!(v.as_f64(), Some(5.0));
        assert_eq!(v.as_str(), None);
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn float_does_not_narrow_to_int() {
        let v = RouteValue::Float(1.5);
        assert_eq!(v.as_f64(), Some(1.5));
        assert_eq!(v.as_i64(), None);
    }

    #[test]
    fn bool_accessors() {
        let v = RouteValue::Bool(true);
        assert_eq!(v.as_bool(), Some(true));
        assert_eq!(v.as_i64(), None);
        assert_eq!(v.as_f64(), None);
        assert_eq!(v.as_str(), None);
    }

    #[test]
    fn unparseable_strings_yield_none() {
        let v = RouteValue::from("abc");
        assert_eq!(v.as_i64(), None);
        assert_eq!(v.as_f64(), None);
        assert_eq!(v.as_bool(), None);
        assert_eq!(RouteValue::from("TRUE").as_bool(), None); // case-sensitive
    }

    #[test]
    fn display_and_to_text() {
        assert_eq!(RouteValue::Int(-9).to_string(), "-9");
        assert_eq!(RouteValue::Float(1.5).to_string(), "1.5");
        assert_eq!(RouteValue::Bool(false).to_string(), "false");
        assert!(matches!(RouteValue::from("x").to_text(), Cow::Borrowed("x")));
        assert!(matches!(RouteValue::Int(1).to_text(), Cow::Owned(_)));
    }

    #[test]
    fn from_impls_and_kind() {
        assert_eq!(RouteValue::from(3_i32), RouteValue::Int(3));
        assert_eq!(RouteValue::from(3_i64).kind(), "int");
        assert_eq!(RouteValue::from(1.0_f64).kind(), "float");
        assert_eq!(RouteValue::from(true).kind(), "bool");
        assert_eq!(RouteValue::from(String::from("s")).kind(), "str");
    }

    #[test]
    fn default_is_empty_str() {
        assert_eq!(RouteValue::default(), RouteValue::Str(String::new()));
    }

    #[test]
    fn clone_and_eq() {
        let a = RouteValue::Float(0.25);
        assert_eq!(a.clone(), a);
        assert_ne!(RouteValue::Int(1), RouteValue::Float(1.0));
    }
}