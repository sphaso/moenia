use std::marker::PhantomData;

pub trait Classifier<E: std::error::Error> {
    fn is_failure(&self, error: &E) -> bool;
}

pub struct AlwaysFailure;
pub struct NeverFailure;
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
