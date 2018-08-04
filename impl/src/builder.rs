#[derive(Clone,Default)]
pub(crate) struct Builder {
    pub(crate) filters: String,
    pub(crate) extactors: String,
}

impl Builder {
    pub(crate) fn append_filter(&mut self, filter: &str) {
        if self.filters.is_empty() {
            self.filters.push('(');
        } else {
            self.filters.push_str(".and(");
        }
        self.filters.push_str(filter);
        self.filters.push(')');
    }

    pub(crate) fn append_closure_args(&mut self, closure_args: &str) {
        if !self.extactors.is_empty() {
            self.extactors.push(',');
        }
        self.extactors.push_str(closure_args);
    }
}
