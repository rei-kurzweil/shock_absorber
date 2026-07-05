const APPROX_CHARS_PER_TOKEN: usize = 4;

#[derive(Clone, Debug)]
pub struct ProviderContextLimits {
    pub provider_name: String,
    pub model_name: String,
    pub max_context_tokens: usize,
    pub reserved_output_tokens: usize,
}

impl ProviderContextLimits {
    pub fn available_input_tokens(&self) -> usize {
        self.max_context_tokens
            .saturating_sub(self.reserved_output_tokens)
    }
}

#[derive(Clone, Debug)]
pub struct BuiltContextSection {
    pub title: String,
    pub estimated_tokens: usize,
    pub used_tokens: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug)]
pub struct BuiltContextWindow {
    pub rendered: String,
    pub limits: ProviderContextLimits,
    pub header_tokens: usize,
    pub used_input_tokens: usize,
    pub truncated: bool,
    pub sections: Vec<BuiltContextSection>,
}

#[derive(Clone, Debug)]
pub struct ContextHeader {
    body: String,
    rendered: String,
    estimated_tokens: usize,
}

impl ContextHeader {
    fn new(body: impl Into<String>) -> Self {
        let body = body.into();
        let rendered = render_header(&body);
        let estimated_tokens = approximate_tokens(&rendered);
        Self {
            body,
            rendered,
            estimated_tokens,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContextSection {
    pub title: String,
    pub body: String,
    pub rendered: String,
    pub estimated_tokens: usize,
}

#[derive(Clone, Debug)]
pub struct LLMContext {
    header: ContextHeader,
    sections: Vec<ContextSection>,
}

impl LLMContext {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: ContextHeader::new(header),
            sections: Vec::new(),
        }
    }

    pub fn child(&self, header: impl Into<String>) -> Self {
        Self::new(header)
    }

    pub fn push_section(&mut self, title: impl Into<String>, body: impl Into<String>) {
        let title = title.into();
        let body = body.into();
        let rendered = render_section_title_and_body(&title, &body);
        let estimated_tokens = approximate_tokens(&rendered);
        self.sections.push(ContextSection {
            title,
            body,
            rendered,
            estimated_tokens,
        });
    }

    pub fn header(&self) -> &str {
        &self.header.body
    }

    pub fn header_rendered(&self) -> &str {
        &self.header.rendered
    }

    pub fn header_tokens(&self) -> usize {
        self.header.estimated_tokens
    }

    pub fn sections(&self) -> &[ContextSection] {
        &self.sections
    }
}

pub fn build_context_window(context: &LLMContext, limits: &ProviderContextLimits) -> String {
    build_context_window_report(context, limits).rendered
}

pub fn build_context_window_report(
    context: &LLMContext,
    limits: &ProviderContextLimits,
) -> BuiltContextWindow {
    let mut rendered = context.header_rendered().to_string();
    let header_tokens = context.header_tokens();
    let mut used_tokens = header_tokens;
    let token_budget = limits.available_input_tokens();
    let mut sections = Vec::new();
    let mut was_truncated = false;

    for section in context.sections() {
        let candidate = &section.rendered;
        let section_tokens = section.estimated_tokens;
        if used_tokens + section_tokens <= token_budget {
            rendered.push_str(candidate);
            used_tokens += section_tokens;
            sections.push(BuiltContextSection {
                title: section.title.clone(),
                estimated_tokens: section_tokens,
                used_tokens: section_tokens,
                truncated: false,
            });
            continue;
        }

        let remaining_tokens = token_budget.saturating_sub(used_tokens);
        if remaining_tokens == 0 {
            was_truncated = true;
            break;
        }

        let truncated_text = truncate_text_to_tokens(candidate, remaining_tokens);
        if !truncated_text.trim().is_empty() {
            let truncated_tokens = approximate_tokens(&truncated_text);
            rendered.push_str(&truncated_text);
            used_tokens += truncated_tokens;
            sections.push(BuiltContextSection {
                title: section.title.clone(),
                estimated_tokens: section_tokens,
                used_tokens: truncated_tokens,
                truncated: true,
            });
        }
        was_truncated = true;
        break;
    }

    BuiltContextWindow {
        rendered,
        limits: limits.clone(),
        header_tokens,
        used_input_tokens: used_tokens,
        truncated: was_truncated,
        sections,
    }
}

pub fn render_header(header: &str) -> String {
    format!("Instructions:\n{header}\n")
}

pub fn render_section(section: &ContextSection) -> String {
    render_section_title_and_body(&section.title, &section.body)
}

fn render_section_title_and_body(title: &str, body: &str) -> String {
    format!("\n## {title}\n{body}\n")
}

pub fn approximate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(APPROX_CHARS_PER_TOKEN)
}

fn truncate_text_to_tokens(text: &str, token_budget: usize) -> String {
    let char_budget = token_budget.saturating_mul(APPROX_CHARS_PER_TOKEN);
    if text.chars().count() <= char_budget {
        return text.to_owned();
    }

    let hard_limit = char_budget.saturating_sub(1);
    let mut cutoff = 0usize;

    for (idx, ch) in text.char_indices() {
        let next = idx + ch.len_utf8();
        if next > hard_limit {
            break;
        }
        cutoff = next;
    }

    let prefix = &text[..cutoff];
    if let Some(double_newline) = prefix.rfind("\n\n") {
        let candidate = prefix[..double_newline].trim_end();
        if !candidate.is_empty() {
            return format!("{candidate}\n\n...");
        }
    }

    if let Some(newline) = prefix.rfind('\n') {
        let candidate = prefix[..newline].trim_end();
        if !candidate.is_empty() {
            return format!("{candidate}\n...");
        }
    }

    let candidate = prefix.trim_end();
    if candidate.is_empty() {
        "...".to_string()
    } else {
        format!("{candidate}...")
    }
}

#[cfg(test)]
mod tests {
    use super::{LLMContext, ProviderContextLimits, build_context_window};

    #[test]
    fn trims_context_to_provider_budget() {
        let mut context = LLMContext::new("brief header");
        context.push_section("first", "a".repeat(32));
        context.push_section("second", "b".repeat(64));

        let limits = ProviderContextLimits {
            provider_name: "test".to_string(),
            model_name: "tiny".to_string(),
            max_context_tokens: 24,
            reserved_output_tokens: 4,
        };

        let rendered = build_context_window(&context, &limits);
        assert!(rendered.contains("## first"));
        assert!(rendered.chars().count() <= 80);
    }

    #[test]
    fn truncates_at_line_boundary_instead_of_mid_line() {
        let mut context = LLMContext::new("brief header");
        context.push_section(
            "collection",
            "item[0]\nuri: one\nauthor: clearsky\nlist_name: alpha\n\nitem[1]\nuri: two\nauthor: clearsky\nlist_name: beta".to_string(),
        );

        let limits = ProviderContextLimits {
            provider_name: "test".to_string(),
            model_name: "tiny".to_string(),
            max_context_tokens: 22,
            reserved_output_tokens: 4,
        };

        let rendered = build_context_window(&context, &limits);
        assert!(rendered.contains("item[0]"));
        assert!(!rendered.contains("list_name: b..."));
        assert!(rendered.ends_with("...\n") || rendered.ends_with("\n..."));
    }
}
