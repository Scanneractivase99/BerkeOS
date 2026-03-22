use regex::Regex;

#[derive(Debug, Clone)]
pub struct Match {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Match {
    pub fn group(&self, _n: usize) -> Option<&str> {
        Some(&self.text)
    }

    pub fn groups(&self) -> Vec<Option<&str>> {
        vec![Some(&self.text)]
    }

    pub fn span(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

pub fn compile(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|e| format!(" regex error: {}", e))
}

pub fn match_(pattern: &str, text: &str) -> Option<Match> {
    let regex = match compile(pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };
    regex.find(text).map(|m| Match {
        text: m.as_str().to_string(),
        start: m.start(),
        end: m.end(),
    })
}

pub fn search(pattern: &str, text: &str) -> Option<Match> {
    let regex = match compile(pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };
    regex.find(text).map(|m| Match {
        text: m.as_str().to_string(),
        start: m.start(),
        end: m.end(),
    })
}

pub fn findall(pattern: &str, text: &str) -> Vec<Match> {
    let regex = match compile(pattern) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    regex
        .find_iter(text)
        .map(|m| Match {
            text: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
        })
        .collect()
}

pub fn sub(pattern: &str, replacement: &str, text: &str) -> String {
    let regex = match compile(pattern) {
        Ok(r) => r,
        Err(_) => return text.to_string(),
    };
    regex.replace_all(text, replacement).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_at_start() {
        let result = match_(r"\d+", "123abc");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.text, "123");
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 3);
    }

    #[test]
    fn test_match_not_at_start() {
        let result = match_(r"\d+", "abc123");
        assert!(result.is_none());
    }

    #[test]
    fn test_search_finds_anywhere() {
        let result = search(r"\d+", "abc123def");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.text, "123");
    }

    #[test]
    fn test_search_no_match() {
        let result = search(r"\d+", "abc");
        assert!(result.is_none());
    }

    #[test]
    fn test_findall_multiple() {
        let result = findall(r"\d+", "a1b2c3");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].text, "1");
        assert_eq!(result[1].text, "2");
        assert_eq!(result[2].text, "3");
    }

    #[test]
    fn test_findall_empty() {
        let result = findall(r"\d+", "abc");
        assert!(result.is_empty());
    }

    #[test]
    fn test_sub_replace() {
        let result = sub(r"\d+", "X", "a1b2c");
        assert_eq!(result, "aXbXc");
    }
}
