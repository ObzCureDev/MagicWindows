use super::{DetectionResult, Layout};

/// Score how well a set of detection results matches a given layout.
///
/// For every detection result we look up the layout's `detectionKeys` entry
/// that has the same `eventCode`.  If the received character matches the
/// entry's `expectedBase`, we award one point.
pub fn score_layout(layout: &Layout, results: &[DetectionResult]) -> u32 {
    let mut score: u32 = 0;
    for result in results {
        for dk in &layout.detection_keys {
            if dk.event_code == result.event_code && dk.expected_base == result.received_char {
                score += 1;
            }
        }
    }
    score
}

/// Return the layout ID with the highest score, or `None` when there is a tie
/// at the top (ambiguous) or no layouts match at all.
pub fn find_best_match(layouts: &[Layout], results: &[DetectionResult]) -> Option<String> {
    if layouts.is_empty() || results.is_empty() {
        return None;
    }

    let mut best_id: Option<String> = None;
    let mut best_score: u32 = 0;
    let mut is_ambiguous = false;

    for layout in layouts {
        let s = score_layout(layout, results);
        if s > best_score {
            best_score = s;
            best_id = Some(layout.id.clone());
            is_ambiguous = false;
        } else if s == best_score && s > 0 {
            // Another layout ties with the current best -> ambiguous.
            is_ambiguous = true;
        }
    }

    if is_ambiguous || best_score == 0 {
        None
    } else {
        best_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::{DetectionKey, DetectionResult, Layout};
    use std::collections::HashMap;

    fn make_layout(id: &str, keys: Vec<(&str, &str)>) -> Layout {
        let detection_keys = keys
            .into_iter()
            .map(|(code, expected)| DetectionKey {
                event_code: code.to_string(),
                prompt: HashMap::new(),
                expected_base: expected.to_string(),
            })
            .collect();

        Layout {
            id: id.to_string(),
            name: HashMap::new(),
            locale: "en-US".to_string(),
            locale_id: "00000409".to_string(),
            dll_name: "test".to_string(),
            description: HashMap::new(),
            detection_keys,
            keys: HashMap::new(),
            dead_keys: HashMap::new(),
        }
    }

    #[test]
    fn perfect_match() {
        let layout = make_layout("fr", vec![("Backquote", "@"), ("Digit8", "!")]);
        let results = vec![
            DetectionResult {
                event_code: "Backquote".into(),
                received_char: "@".into(),
            },
            DetectionResult {
                event_code: "Digit8".into(),
                received_char: "!".into(),
            },
        ];
        assert_eq!(score_layout(&layout, &results), 2);
    }

    #[test]
    fn no_match() {
        let layout = make_layout("fr", vec![("Backquote", "@")]);
        let results = vec![DetectionResult {
            event_code: "Backquote".into(),
            received_char: "`".into(),
        }];
        assert_eq!(score_layout(&layout, &results), 0);
    }

    #[test]
    fn best_match_single() {
        let l1 = make_layout("fr", vec![("Backquote", "@"), ("Digit8", "!")]);
        let l2 = make_layout("de", vec![("Backquote", "^"), ("Digit8", "8")]);
        let results = vec![
            DetectionResult {
                event_code: "Backquote".into(),
                received_char: "@".into(),
            },
            DetectionResult {
                event_code: "Digit8".into(),
                received_char: "!".into(),
            },
        ];
        assert_eq!(
            find_best_match(&[l1, l2], &results),
            Some("fr".to_string())
        );
    }

    #[test]
    fn ambiguous_returns_none() {
        let l1 = make_layout("a", vec![("Backquote", "@")]);
        let l2 = make_layout("b", vec![("Backquote", "@")]);
        let results = vec![DetectionResult {
            event_code: "Backquote".into(),
            received_char: "@".into(),
        }];
        assert_eq!(find_best_match(&[l1, l2], &results), None);
    }
}
