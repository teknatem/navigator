use std::fs;
use std::path::Path;

pub struct GitignoreParser {
    patterns: Vec<GitignorePattern>,
}

#[derive(Debug)]
struct GitignorePattern {
    pattern: String,
    is_negation: bool,
    is_directory_only: bool,
    is_rooted: bool,
}

impl GitignoreParser {
    pub fn from_file(gitignore_path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(gitignore_path)
            .map_err(|e| format!("Failed to read .gitignore: {}", e))?;
        
        let patterns = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|line| Self::parse_pattern(line))
            .collect();
        
        Ok(Self { patterns })
    }
    
    fn parse_pattern(line: &str) -> Option<GitignorePattern> {
        let (is_negation, line) = if let Some(rest) = line.strip_prefix('!') {
            (true, rest)
        } else {
            (false, line)
        };
        
        let (is_directory_only, line) = if let Some(rest) = line.strip_suffix('/') {
            (true, rest)
        } else {
            (false, line)
        };
        
        let (is_rooted, pattern) = if let Some(rest) = line.strip_prefix('/') {
            (true, rest.to_string())
        } else {
            (false, line.to_string())
        };
        
        if pattern.is_empty() {
            return None;
        }
        
        Some(GitignorePattern {
            pattern,
            is_negation,
            is_directory_only,
            is_rooted,
        })
    }
    
    pub fn is_ignored(&self, path: &str, is_directory: bool) -> bool {
        let mut ignored = false;
        
        for pattern in &self.patterns {
            if pattern.is_directory_only && !is_directory {
                continue;
            }
            
            if self.matches_pattern(&pattern.pattern, path, pattern.is_rooted) {
                ignored = !pattern.is_negation;
            }
        }
        
        ignored
    }
    
    fn matches_pattern(&self, pattern: &str, path: &str, is_rooted: bool) -> bool {
        let path_normalized = path.replace('\\', "/");
        
        if is_rooted {
            // Rooted patterns match from the beginning
            return self.glob_match(pattern, &path_normalized);
        }
        
        // Non-rooted patterns can match anywhere in the path
        if self.glob_match(pattern, &path_normalized) {
            return true;
        }
        
        // Try matching against any component of the path
        for component in path_normalized.split('/') {
            if self.glob_match(pattern, component) {
                return true;
            }
        }
        
        false
    }
    
    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        // Simple glob matching supporting * and **
        if pattern.contains("**") {
            // Handle ** patterns (matches any number of directories)
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1].trim_start_matches('/');
                
                if !prefix.is_empty() && !text.starts_with(prefix) {
                    return false;
                }
                
                if !suffix.is_empty() && !text.ends_with(suffix) {
                    return false;
                }
                
                return true;
            }
        }
        
        // Simple * wildcard matching
        let pattern_parts: Vec<&str> = pattern.split('*').collect();
        let mut pos = 0;
        
        for (i, part) in pattern_parts.iter().enumerate() {
            if i == 0 {
                // First part must match at the beginning
                if !text[pos..].starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == pattern_parts.len() - 1 {
                // Last part must match at the end
                if !text[pos..].ends_with(part) {
                    return false;
                }
            } else {
                // Middle parts must exist somewhere
                if let Some(idx) = text[pos..].find(part) {
                    pos += idx + part.len();
                } else {
                    return false;
                }
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_patterns() {
        let parser = GitignoreParser {
            patterns: vec![
                GitignorePattern {
                    pattern: "target".to_string(),
                    is_negation: false,
                    is_directory_only: true,
                    is_rooted: false,
                },
                GitignorePattern {
                    pattern: "*.exe".to_string(),
                    is_negation: false,
                    is_directory_only: false,
                    is_rooted: false,
                },
            ],
        };
        
        assert!(parser.is_ignored("target", true));
        assert!(!parser.is_ignored("target", false));
        assert!(parser.is_ignored("app.exe", false));
    }
}

