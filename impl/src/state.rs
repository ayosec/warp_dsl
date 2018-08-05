use std::mem;

#[derive(Clone,Default)]
pub(crate) struct State {
    pub(crate) filters: String,
    pub(crate) closure_args: String,
    pub(crate) http_method: Option<String>,
}

impl State {
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

    pub(crate) fn append_http_method(&mut self, http_method: &str) {
        let old = mem::replace(&mut self.http_method, Some(http_method.into()));
        if let Some(m) = old {
            panic!("Found HTTP method '{}' when try to set '{}'", m, http_method);
        }
    }
}
