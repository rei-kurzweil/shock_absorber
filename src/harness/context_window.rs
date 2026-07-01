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
pub struct ContextSection {
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug)]
pub struct LLMContext {
    header: String,
    sections: Vec<ContextSection>,
}

impl LLMContext {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            sections: Vec::new(),
        }
    }

    pub fn child(&self, header: impl Into<String>) -> Self {
        Self::new(header)
    }

    pub fn push_section(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.sections.push(ContextSection {
            title: title.into(),
            body: body.into(),
        });
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub fn sections(&self) -> &[ContextSection] {
        &self.sections
    }
}

pub fn build_context_window(context: &LLMContext, limits: &ProviderContextLimits) -> String {
    let mut rendered = render_header(context.header());
    let mut used_tokens = approximate_tokens(&rendered);
    let token_budget = limits.available_input_tokens();

    for section in context.sections() {
        let candidate = render_section(section);
        let section_tokens = approximate_tokens(&candidate);
        if used_tokens + section_tokens <= token_budget {
            rendered.push_str(&candidate);
            used_tokens += section_tokens;
            continue;
        }

        let remaining_tokens = token_budget.saturating_sub(used_tokens);
        if remaining_tokens == 0 {
            break;
        }

        let truncated = truncate_text_to_tokens(&candidate, remaining_tokens);
        if !truncated.trim().is_empty() {
            rendered.push_str(&truncated);
        }
        break;
    }

    rendered
}

fn render_header(header: &str) -> String {
    format!("System:\n{header}\n")
}

fn render_section(section: &ContextSection) -> String {
    format!("\n## {}\n{}\n", section.title, section.body)
}

fn approximate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(APPROX_CHARS_PER_TOKEN)
}

fn truncate_text_to_tokens(text: &str, token_budget: usize) -> String {
    let char_budget = token_budget.saturating_mul(APPROX_CHARS_PER_TOKEN);
    if text.chars().count() <= char_budget {
        return text.to_owned();
    }

    let mut truncated = String::new();
    for ch in text.chars().take(char_budget.saturating_sub(1)) {
        truncated.push(ch);
    }
    truncated.push_str("...");
    truncated
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
}
