use crate::model::{NativeMode, ParsedChargeTypes};

pub fn parse_charge_types(input: &str) -> std::result::Result<ParsedChargeTypes, String> {
    let chars: Vec<char> = input.trim().chars().collect();
    let mut index = 0;
    let mut modes = Vec::new();
    let mut active_raw = None;

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }

        if index >= chars.len() {
            break;
        }

        if chars[index] == '[' {
            index += 1;
            let start = index;
            while index < chars.len() && chars[index] != ']' {
                index += 1;
            }
            if index >= chars.len() {
                return Err("missing closing bracket for active native mode".to_string());
            }

            let raw = chars[start..index]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
            if raw.is_empty() {
                return Err("active native mode is empty".to_string());
            }
            index += 1;

            if active_raw.is_some() {
                return Err("multiple active native modes found".to_string());
            }
            active_raw = Some(raw.clone());
            modes.push(NativeMode {
                label: display_label(&raw),
                raw,
            });
        } else {
            let start = index;
            while index < chars.len() && !chars[index].is_whitespace() {
                if chars[index] == ']' {
                    return Err("unexpected closing bracket in native modes".to_string());
                }
                index += 1;
            }

            let raw = chars[start..index].iter().collect::<String>();
            if raw.contains('[') {
                return Err("unexpected opening bracket inside native mode token".to_string());
            }
            if !raw.is_empty() {
                modes.push(NativeMode {
                    label: display_label(&raw),
                    raw,
                });
            }
        }
    }

    if modes.is_empty() {
        return Err("native mode list is empty".to_string());
    }

    Ok(ParsedChargeTypes { modes, active_raw })
}

pub fn normalize_mode(mode: &str) -> String {
    mode.trim()
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub fn display_label(raw: &str) -> String {
    match normalize_mode(raw).as_str() {
        "fast" => "Fast Charge".to_string(),
        "standard" => "Standard".to_string(),
        "long life" => "Long Life".to_string(),
        "adaptive" => "Adaptive".to_string(),
        "custom" => "Custom".to_string(),
        "trickle" => "Trickle".to_string(),
        "bypass" => "Bypass".to_string(),
        _ => raw.replace('_', " "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bracketed_active_first() {
        let parsed = parse_charge_types("[Fast] Standard Long_Life").unwrap();

        assert_eq!(parsed.active_raw.as_deref(), Some("Fast"));
        assert_eq!(raw_modes(&parsed), ["Fast", "Standard", "Long_Life"]);
    }

    #[test]
    fn parses_bracketed_active_middle() {
        let parsed = parse_charge_types("Fast [Standard] Long_Life").unwrap();

        assert_eq!(parsed.active_raw.as_deref(), Some("Standard"));
        assert_eq!(raw_modes(&parsed), ["Fast", "Standard", "Long_Life"]);
    }

    #[test]
    fn parses_bracketed_active_last() {
        let parsed = parse_charge_types("Fast Standard [Long_Life]").unwrap();

        assert_eq!(parsed.active_raw.as_deref(), Some("Long_Life"));
        assert_eq!(raw_modes(&parsed), ["Fast", "Standard", "Long_Life"]);
    }

    #[test]
    fn preserves_spaces_inside_bracketed_mode() {
        let parsed = parse_charge_types("Fast [Long Life]").unwrap();

        assert_eq!(parsed.active_raw.as_deref(), Some("Long Life"));
        assert_eq!(raw_modes(&parsed), ["Fast", "Long Life"]);
    }

    #[test]
    fn normalizes_case_spaces_and_underscores_for_matching() {
        assert_eq!(normalize_mode("Long_Life"), normalize_mode(" long   life "));
        assert_eq!(normalize_mode("FAST"), normalize_mode("Fast"));
        assert_eq!(display_label("Long_Life"), "Long Life");
    }

    #[test]
    fn rejects_unclosed_active_mode() {
        let error = parse_charge_types("Fast [Long Life").unwrap_err();

        assert!(error.contains("missing closing bracket"));
    }

    fn raw_modes(parsed: &ParsedChargeTypes) -> Vec<&str> {
        parsed.modes.iter().map(|mode| mode.raw.as_str()).collect()
    }
}
