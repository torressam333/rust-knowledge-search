pub fn tokenize(text: &str) -> Vec<String> {
    // 1. convert text to lower case
    let lower = text.to_lowercase();

    // 2. create a new empty String buffer
    let mut cleaned = String::new();

    for ch in lower.chars() {
        // 3. if char is ASCII alphanumeric or whitespace the push char
        if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
            cleaned.push(ch);
        } else {
            // 4. Otherwise replace punctuation with white space
            cleaned.push(' '); // replace punctuation or non ascii chars
        }
    }

    // split into tokens
    cleaned.split_whitespace().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation() {
        let tokens = tokenize("Rust!!! is... awesome??");
        assert_eq!(tokens, vec!["rust", "is", "awesome"]);
    }

    #[test]
    fn test_lowercasing() {
        let tokens = tokenize("HeLLo WoRLD");

        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_unicode_behavior() {
        let tokens = tokenize("naïve café");
        assert_eq!(tokens, vec!["na", "ve", "caf"]);
    }
}
