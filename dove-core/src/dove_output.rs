pub trait DoveOutput {
    fn print(&self, message: String);
    fn warning(&self, message: String);
    fn error(&self, message: String);
}
