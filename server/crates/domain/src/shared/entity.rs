// Also require Eq ?
pub trait Entity {
    fn id(&self) -> String;
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
