use std::fmt;

use serde::Serialize;

/// The semantic status of a diagnostic node.
///
/// `Satisfied`, `Unavailable`, and `Impossible` mirror rust-miniscript's
/// witness states. A missing signature is `Impossible`; a missing preimage
/// or an unmet timelock is `Unavailable`.
///
/// `Unsupported` indicates a fragment outside the supported scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Satisfied,
    Unavailable,
    Impossible,
    Unsupported,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Status::Satisfied => "SATISFIED",
            Status::Unavailable => "UNAVAILABLE",
            Status::Impossible => "IMPOSSIBLE",
            Status::Unsupported => "UNSUPPORTED",
        };
        f.write_str(status)
    }
}

impl Status {
    /// AND semantics: `Impossible` > `Unsupported` > `Unavailable` >
    /// `Satisfied`.
    pub fn combine_and(left: Status, right: Status) -> Status {
        use Status::*;
        match (left, right) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Unsupported, _) | (_, Unsupported) => Unsupported,
            (Unavailable, _) | (_, Unavailable) => Unavailable,
            (Satisfied, Satisfied) => Satisfied,
        }
    }

    /// OR semantics: `Satisfied` > `Unsupported` > `Unavailable` >
    /// `Impossible`.
    pub fn combine_or(left: Status, right: Status) -> Status {
        use Status::*;
        match (left, right) {
            (Satisfied, _) | (_, Satisfied) => Satisfied,
            (Unsupported, _) | (_, Unsupported) => Unsupported,
            (Unavailable, _) | (_, Unavailable) => Unavailable,
            (Impossible, Impossible) => Impossible,
        }
    }

    /// Determines whether a threshold is satisfied, unavailable, impossible,
    /// or unsupported based on the current child statuses.
    pub fn combine_thresh(k: usize, statuses: &[Status]) -> Status {
        let satisfied = statuses
            .iter()
            .filter(|status| **status == Status::Satisfied)
            .count();
        let unavailable = statuses
            .iter()
            .filter(|status| **status == Status::Unavailable)
            .count();
        let unsupported = statuses
            .iter()
            .filter(|status| **status == Status::Unsupported)
            .count();

        if satisfied >= k {
            return Status::Satisfied;
        }

        let best_case = satisfied + unavailable + unsupported;
        if best_case < k {
            return Status::Impossible;
        }

        if unsupported > 0 {
            return Status::Unsupported;
        }

        Status::Unavailable
    }
}

/// A node in the diagnostic tree.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Short label such as `"pk(<hex>)"` or `"older(144)"`.
    pub fragment: String,
    pub status: Status,
    pub reason: Option<String>,
    pub metadata: Vec<(String, String)>,
    pub children: Vec<Diagnostic>,
}

impl Diagnostic {
    pub fn leaf(fragment: impl Into<String>, status: Status, reason: impl Into<String>) -> Self {
        Diagnostic {
            fragment: fragment.into(),
            status,
            reason: Some(reason.into()),
            metadata: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn combinator(
        fragment: impl Into<String>,
        status: Status,
        children: Vec<Diagnostic>,
    ) -> Self {
        Diagnostic {
            fragment: fragment.into(),
            status,
            reason: None,
            metadata: Vec::new(),
            children,
        }
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn is_satisfied(&self) -> bool {
        matches!(self.status, Status::Satisfied)
    }

    /// Renders the diagnostic tree as human-readable text.
    pub fn render(&self) -> String {
        let mut output = String::new();
        self.render_root(&mut output);
        output
    }

    fn render_root(&self, output: &mut String) {
        output.push_str(&self.header_label());
        output.push('\n');
        self.render_body(output, "");
    }

    fn header_label(&self) -> String {
        if self.children.is_empty() {
            self.fragment.clone()
        } else {
            format!("{} [{}]", self.fragment, self.status)
        }
    }

    fn render_body(&self, output: &mut String, prefix: &str) {
        let detail_prefix = format!("{prefix}    ");

        if self.children.is_empty() {
            output.push_str(prefix);
            output.push_str("└── ");
            output.push_str(&self.status.to_string());
            output.push('\n');

            if let Some(reason) = &self.reason {
                output.push_str(&detail_prefix);
                output.push_str(reason);
                output.push('\n');
            }
        }

        for (key, value) in &self.metadata {
            output.push_str(&detail_prefix);
            output.push_str(key);
            output.push_str(": ");
            output.push_str(value);
            output.push('\n');
        }

        let child_count = self.children.len();

        for (index, child) in self.children.iter().enumerate() {
            let is_last = index + 1 == child_count;
            let connector = if is_last { "└── " } else { "├── " };

            output.push_str(prefix);
            output.push_str(connector);
            output.push_str(&child.header_label());
            output.push('\n');

            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };

            child.render_body(output, &child_prefix);

            if !is_last {
                output.push_str(&child_prefix);
                output.push('\n');
            }
        }
    }
}
