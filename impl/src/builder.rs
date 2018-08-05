#[derive(Clone,Default)]
pub(crate) struct Builder {
    pub(crate) filters: String,
    pub(crate) closure_args: String,
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
        if !self.closure_args.is_empty() {
            self.closure_args.push(',');
        }
        self.closure_args.push_str(closure_args);
    }
}
