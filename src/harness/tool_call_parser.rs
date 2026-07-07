use serde_json::{Map, Number, Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromptToolCall {
    pub name: String,
    pub args: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedToolCallBlock {
    pub accepted_block: String,
    pub discarded_trailing: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolCallParseError {
    NoToolCallPresent,
    LeadingProseBeforeToolCall,
    MissingToolName,
    MissingArgs,
    ArgsNotParseableAsJsonOrYaml,
}

impl ToolCallParseError {
    pub fn diagnostic(&self) -> &'static str {
        match self {
            Self::NoToolCallPresent => "no TOOL_CALL block was present",
            Self::LeadingProseBeforeToolCall => {
                "strict internal mode requires TOOL_CALL as the first emitted block"
            }
            Self::MissingToolName => "TOOL_CALL block is missing a name: field",
            Self::MissingArgs => "TOOL_CALL block is missing an args: payload",
            Self::ArgsNotParseableAsJsonOrYaml => {
                "TOOL_CALL args were not parseable as JSON or the supported YAML-like mapping format"
            }
        }
    }
}

pub fn parse_prompt_tool_call(response: &str) -> Result<PromptToolCall, ToolCallParseError> {
    let mut lines = response.lines();
    while let Some(line) = lines.next() {
        let trimmed_line = line.trim();
        if trimmed_line != "TOOL_CALL"
            && !trimmed_line.ends_with("TOOL_CALL")
            && !trimmed_line.contains("TOOL_CALL")
        {
            continue;
        }

        let mut name = None;
        let mut args_lines = Vec::new();

        for line in lines.by_ref() {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("name:") {
                name = Some(value.trim().to_string());
                continue;
            }

            if let Some(value) = trimmed.strip_prefix("args:") {
                args_lines.push(value.to_string());
                args_lines.extend(lines.by_ref().map(str::to_owned));
                break;
            }
        }

        let name = name.ok_or(ToolCallParseError::MissingToolName)?;
        let args = parse_tool_args(&args_lines.join("\n"))?;
        return Ok(PromptToolCall { name, args });
    }

    Err(ToolCallParseError::NoToolCallPresent)
}

pub fn extract_leading_tool_call_block(
    response: &str,
) -> Result<AcceptedToolCallBlock, ToolCallParseError> {
    let trimmed = response.trim();
    let tool_call_start = trimmed
        .find("TOOL_CALL")
        .ok_or(ToolCallParseError::NoToolCallPresent)?;
    if !trimmed[..tool_call_start].trim().is_empty() {
        return Err(ToolCallParseError::LeadingProseBeforeToolCall);
    }

    let after = &trimmed[tool_call_start..];
    let name_pos = after
        .find("\nname:")
        .ok_or(ToolCallParseError::MissingToolName)?;
    let args_pos = after.find("\nargs:").ok_or(ToolCallParseError::MissingArgs)?;
    if args_pos <= name_pos {
        return Err(ToolCallParseError::MissingArgs);
    }

    let args_section = &after[args_pos + "\nargs:".len()..];
    let args_len = extract_args_span(args_section)?;
    let block_end = args_pos + "\nargs:".len() + args_len;
    let accepted_block = after[..block_end].trim().to_string();
    let trailing = after[block_end..].trim();

    Ok(AcceptedToolCallBlock {
        accepted_block,
        discarded_trailing: (!trailing.is_empty()).then(|| trailing.to_string()),
    })
}

pub fn parse_tool_args(raw_args: &str) -> Result<Value, ToolCallParseError> {
    if let Some(raw_json) = extract_first_args_object(raw_args) {
        return parse_tool_args_json(&raw_json)
            .or_else(|| parse_yaml_like_args(raw_args))
            .ok_or(ToolCallParseError::ArgsNotParseableAsJsonOrYaml);
    }

    parse_yaml_like_args(raw_args).ok_or(ToolCallParseError::ArgsNotParseableAsJsonOrYaml)
}

fn extract_args_span(raw_args: &str) -> Result<usize, ToolCallParseError> {
    if let Some(raw_json) = extract_first_args_object(raw_args) {
        let start = raw_args
            .find(&raw_json)
            .ok_or(ToolCallParseError::ArgsNotParseableAsJsonOrYaml)?;
        return Ok(start + raw_json.len());
    }

    let yaml_block = extract_yaml_like_block(raw_args).ok_or(ToolCallParseError::MissingArgs)?;
    let start = raw_args
        .find(&yaml_block)
        .ok_or(ToolCallParseError::ArgsNotParseableAsJsonOrYaml)?;
    Ok(start + yaml_block.len())
}

fn extract_first_args_object(raw_args: &str) -> Option<String> {
    let chars = raw_args.char_indices().collect::<Vec<_>>();
    let start = chars.iter().find(|(_, ch)| *ch == '{').map(|(idx, _)| *idx)?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in raw_args[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            continue;
        }

        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                let end = start + idx + ch.len_utf8();
                return Some(raw_args[start..end].to_string());
            }
        }
    }

    None
}

fn parse_tool_args_json(raw_args: &str) -> Option<Value> {
    serde_json::from_str(raw_args)
        .ok()
        .or_else(|| serde_json::from_str(&repair_tool_args_json(raw_args)).ok())
}

fn repair_tool_args_json(raw_args: &str) -> String {
    let normalized_quotes = raw_args.replace("<|\"|>", "\"");
    quote_bare_object_keys(&normalized_quotes)
}

fn quote_bare_object_keys(input: &str) -> String {
    let chars = input.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while i < chars.len() {
        let ch = chars[i];

        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }

        if ch == '{' || ch == ',' {
            out.push(ch);
            i += 1;

            while i < chars.len() && chars[i].is_whitespace() {
                out.push(chars[i]);
                i += 1;
            }

            let key_start = i;
            if i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '_') {
                i += 1;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let key_end = i;
                let mut probe = i;
                while probe < chars.len() && chars[probe].is_whitespace() {
                    probe += 1;
                }

                if probe < chars.len() && chars[probe] == ':' {
                    out.push('"');
                    for key_char in &chars[key_start..key_end] {
                        out.push(*key_char);
                    }
                    out.push('"');
                    while i < probe {
                        out.push(chars[i]);
                        i += 1;
                    }
                    continue;
                }

                for key_char in &chars[key_start..key_end] {
                    out.push(*key_char);
                }
                continue;
            }

            continue;
        }

        out.push(ch);
        i += 1;
    }

    out
}

fn parse_yaml_like_args(raw_args: &str) -> Option<Value> {
    let block = extract_yaml_like_block(raw_args)?;
    let mut object = Map::new();

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let (key, raw_value) = trimmed.split_once(':')?;
        let key = key.trim();
        if key.is_empty() {
            return None;
        }

        let raw_value = raw_value.trim();
        if raw_value.is_empty() {
            return None;
        }

        object.insert(key.to_string(), parse_yaml_like_scalar(raw_value)?);
    }

    Some(Value::Object(object))
}

fn extract_yaml_like_block(raw_args: &str) -> Option<String> {
    let mut collected = Vec::new();
    let mut started = false;

    for line in raw_args.lines() {
        if !started {
            if line.trim().is_empty() {
                continue;
            }

            if line.starts_with(' ') || line.starts_with('\t') {
                started = true;
                collected.push(line.to_string());
                continue;
            }

            if !line.trim().starts_with('{') {
                return Some(line.to_string());
            }

            return None;
        }

        if line.trim().is_empty() {
            break;
        }

        if line.starts_with(' ') || line.starts_with('\t') {
            collected.push(line.to_string());
            continue;
        }

        break;
    }

    (!collected.is_empty()).then(|| collected.join("\n"))
}

fn parse_yaml_like_scalar(raw_value: &str) -> Option<Value> {
    let normalized = raw_value.replace("<|\"|>", "\"");
    if normalized.starts_with('"') && normalized.ends_with('"') && normalized.len() >= 2 {
        return serde_json::from_str::<String>(&normalized)
            .ok()
            .map(Value::String);
    }

    if normalized.starts_with('\'') && normalized.ends_with('\'') && normalized.len() >= 2 {
        return Some(Value::String(normalized[1..normalized.len() - 1].to_string()));
    }

    match normalized.as_str() {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        "null" => return Some(Value::Null),
        _ => {}
    }

    if let Ok(int_value) = normalized.parse::<i64>() {
        return Some(Value::Number(Number::from(int_value)));
    }

    if let Ok(uint_value) = normalized.parse::<u64>() {
        return Some(Value::Number(Number::from(uint_value)));
    }

    if let Ok(float_value) = normalized.parse::<f64>() {
        return Number::from_f64(float_value).map(Value::Number);
    }

    Some(Value::String(normalized))
}

#[cfg(test)]
mod tests {
    use super::{ToolCallParseError, extract_leading_tool_call_block, parse_prompt_tool_call};
    use serde_json::json;

    #[test]
    fn parses_prompt_tool_call_block() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: llm_search\nargs: {\"query\":\"who is rei-cast.xyz?\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(tool_call.args["query"], "who is rei-cast.xyz?");
    }

    #[test]
    fn parses_prompt_tool_call_block_with_channel_prefix() {
        let tool_call = parse_prompt_tool_call(
            "<|channel>thought\nplan text\n<channel|>TOOL_CALL\nname: list_collections\nargs: {\"actor_did\":\"did:plc:testactor\"}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "list_collections");
        assert_eq!(tool_call.args["actor_did"], "did:plc:testactor");
    }

    #[test]
    fn parses_prompt_tool_call_block_with_repaired_args_json() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: llm_search\nargs:\n{query:<|\"|>what are people on Bluesky saying about topic x<|\"|>}",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "llm_search");
        assert_eq!(
            tool_call.args["query"],
            "what are people on Bluesky saying about topic x"
        );
    }

    #[test]
    fn parses_prompt_tool_call_block_with_yaml_like_args() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: hydrate_actor_scope\nargs:\n  actor_did: did:plc:frudpt5kpurby7s7qdaz7zyw\n  include_recent_posts: true\n  include_recent_replies_received: true",
        )
        .expect("expected tool call");

        assert_eq!(tool_call.name, "hydrate_actor_scope");
        assert_eq!(
            tool_call.args,
            json!({
                "actor_did": "did:plc:frudpt5kpurby7s7qdaz7zyw",
                "include_recent_posts": true,
                "include_recent_replies_received": true,
            })
        );
    }

    #[test]
    fn parses_first_tool_call_when_trailing_thought_continues() {
        let tool_call = parse_prompt_tool_call(
            "TOOL_CALL\nname: list_collections\nargs: {actor_did: \"did:plc:testactor\"}\n\n<|channel>thought\nextra commentary\n<channel|>TOOL_CALL\nname: llm_search\nargs: {query:\"who is rei-cast.xyz?\"}",
        )
        .expect("expected first tool call");

        assert_eq!(tool_call.name, "list_collections");
        assert_eq!(tool_call.args["actor_did"], "did:plc:testactor");
    }

    #[test]
    fn extracts_leading_yaml_like_tool_call_and_discards_trailing_output() {
        let accepted = extract_leading_tool_call_block(
            "TOOL_CALL\nname: collection_search\nargs:\n  collection_id: clearsky_lists:did:plc:testactor\n  prompt: check lists\n\nSelf-correction: hypothetical results go here",
        )
        .expect("expected accepted block");

        assert!(accepted.accepted_block.contains("collection_id"));
        assert!(
            accepted
                .discarded_trailing
                .as_deref()
                .is_some_and(|trailing| trailing.contains("Self-correction"))
        );
    }

    #[test]
    fn rejects_leading_prose_before_tool_call() {
        let err = extract_leading_tool_call_block(
            "I will search now.\n\nTOOL_CALL\nname: collection_search\nargs: {\"collection_id\":\"clearsky_lists:did:plc:testactor\",\"prompt\":\"check lists\"}",
        )
        .expect_err("expected leading prose rejection");

        assert_eq!(err, ToolCallParseError::LeadingProseBeforeToolCall);
    }
}
