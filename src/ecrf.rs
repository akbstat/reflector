pub trait ECRF {
    fn form_page(&self, form: &str) -> Option<usize>;
}
