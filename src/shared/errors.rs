#[derive(Debug)]
pub struct NotFoundError;

impl Error for NotFoundError {}

impl std::fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Could not find them item requested.")
    }
}