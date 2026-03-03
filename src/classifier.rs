use std::marker::PhantomData;

/// Determines whether an error should count as a failure for the circuit breaker.
///
/// Implement this trait to create custom error classification strategies.
/// Three built-in implementations are provided: [`AlwaysFailure`], [`NeverFailure`],
/// and [`MatchClassifier`].
pub trait Classifier<E: std::error::Error> {
    fn is_failure(&self, error: &E) -> bool;
}

/// A classifier that always counts errors as failures.
///
/// Useful for testing or when every error should trip the breaker.
///
/// # Example
///
/// ```rust
/// use moenia::AlwaysFailure;
///
/// let classifier = AlwaysFailure;
/// ```
pub struct AlwaysFailure;

/// A classifier that never counts errors as failures.
///
/// Useful for testing or when you want to disable circuit breaking temporarily.
///
/// # Example
///
/// ```rust
/// use moenia::NeverFailure;
///
/// let classifier = NeverFailure;
/// ```
pub struct NeverFailure;

/// A classifier that uses a closure to determine if an error is a failure.
///
/// This is the most flexible built-in classifier — use it when you need
/// to classify errors based on their content.
///
/// # Example
///
/// ```rust
/// use moenia::MatchClassifier;
///
/// let classifier = MatchClassifier::new(|e: &std::io::Error| {
///     e.kind() == std::io::ErrorKind::TimedOut
/// });
/// ```
pub struct MatchClassifier<E, F: Fn(&E) -> bool> {
    matcher: F,
    _phantom: PhantomData<E>,
}

impl<E: std::error::Error, F: Fn(&E) -> bool> MatchClassifier<E, F> {
    pub fn new(matcher: F) -> Self {
        MatchClassifier {
            matcher,
            _phantom: PhantomData,
        }
    }
}

impl<E: std::error::Error> Classifier<E> for AlwaysFailure {
    fn is_failure(&self, _error: &E) -> bool {
        true
    }
}

impl<E: std::error::Error> Classifier<E> for NeverFailure {
    fn is_failure(&self, _error: &E) -> bool {
        false
    }
}

impl<E: std::error::Error, F: Fn(&E) -> bool> Classifier<E> for MatchClassifier<E, F> {
    fn is_failure(&self, error: &E) -> bool {
        (self.matcher)(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn always_failure_error_test() {
        let classifier = AlwaysFailure;
        assert!(classifier.is_failure(&std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error"
        )));
    }

    #[test]
    fn never_failure_error_test() {
        let classifier = NeverFailure;
        assert!(!classifier.is_failure(&std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error"
        )));
    }

    #[test]
    fn match_classifier_error_test() {
        let classifier =
            MatchClassifier::new(|e: &std::io::Error| e.kind() == std::io::ErrorKind::Other);
        assert!(classifier.is_failure(&std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error"
        )));
    }

    #[test]
    fn match_classifier_ok_test() {
        let classifier =
            MatchClassifier::new(|e: &std::io::Error| e.kind() != std::io::ErrorKind::Other);
        assert!(!classifier.is_failure(&std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error"
        )));
    }
}
