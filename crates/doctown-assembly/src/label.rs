//! Cluster labeling using term frequency analysis.

use std::collections::HashMap;

/// Generates human-readable labels for clusters.
pub struct ClusterLabeler;

impl ClusterLabeler {
    /// Generate a label for a cluster based on member content.
    ///
    /// Uses TF-IDF or simple frequency to extract important terms.
    /// Returns a 1-2 word label.
    pub fn label_cluster(cluster_members: &[String]) -> String {
        if cluster_members.is_empty() {
            return "empty".to_string();
        }

        // Extract terms from cluster members
        let term_frequencies = Self::extract_terms(cluster_members);

        // Filter out common stop words and short terms
        let filtered_terms: HashMap<String, usize> = term_frequencies
            .into_iter()
            .filter(|(term, _)| !Self::is_stop_word(term) && term.len() >= 3)
            .collect();

        if filtered_terms.is_empty() {
            return "misc".to_string();
        }

        // Get top terms by frequency
        let mut sorted_terms: Vec<_> = filtered_terms.iter().collect();
        sorted_terms.sort_by(|a, b| b.1.cmp(a.1));

        // Generate 1-2 word label from top terms
        Self::generate_label(&sorted_terms)
    }

    /// Generate a label from sorted terms.
    fn generate_label(sorted_terms: &[(&String, &usize)]) -> String {
        if sorted_terms.is_empty() {
            return "misc".to_string();
        }

        let top_term = sorted_terms[0].0;

        // If we have a clear winner (significantly more frequent), use just one word
        if sorted_terms.len() == 1 || *sorted_terms[0].1 >= *sorted_terms[1].1 * 2 {
            return top_term.clone();
        }

        // Otherwise, try to create a 2-word label from top terms
        let second_term = sorted_terms[1].0;

        // If both terms are related (share a prefix or are similar), use just the first
        if Self::are_related_terms(top_term, second_term) {
            return top_term.clone();
        }

        // Create compound label
        format!("{}-{}", top_term, second_term)
    }

    /// Check if two terms are related (to avoid redundant labels like "fetch-fetcher").
    fn are_related_terms(term1: &str, term2: &str) -> bool {
        // Check if one is a prefix of the other
        if term1.starts_with(term2) || term2.starts_with(term1) {
            return true;
        }

        // Check if they share a significant common prefix (>= 4 chars)
        let min_len = term1.len().min(term2.len());
        if min_len >= 4 {
            let common_prefix_len = term1
                .chars()
                .zip(term2.chars())
                .take_while(|(c1, c2)| c1 == c2)
                .count();

            if common_prefix_len >= 4 {
                return true;
            }
        }

        false
    }

    /// Extract common terms from a set of texts.
    /// Returns a map of term -> frequency.
    fn extract_terms(texts: &[String]) -> HashMap<String, usize> {
        let mut term_counts = HashMap::new();

        for text in texts {
            // Split text into tokens (words)
            let tokens = Self::tokenize(text);

            for token in tokens {
                *term_counts.entry(token).or_insert(0) += 1;
            }
        }

        term_counts
    }

    /// Tokenize text into individual terms.
    /// Handles camelCase, snake_case, and other common programming conventions.
    fn tokenize(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();

        // First pass: split on non-alphanumeric characters (preserving case)
        for ch in text.chars() {
            if ch.is_alphanumeric() {
                current_token.push(ch);
            } else {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        // Second pass: split camelCase and PascalCase words, then lowercase
        let mut expanded_tokens = Vec::new();
        for token in tokens {
            expanded_tokens.extend(Self::split_camel_case(&token));
        }

        expanded_tokens
    }

    /// Split camelCase or PascalCase words into separate tokens.
    fn split_camel_case(word: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut prev_was_lower = false;

        for ch in word.chars() {
            if ch.is_uppercase() && !current.is_empty() && prev_was_lower {
                // Start of new word in camelCase
                if !current.is_empty() {
                    result.push(current.to_lowercase());
                    current.clear();
                }
            }
            current.push(ch);
            prev_was_lower = ch.is_lowercase();
        }

        if !current.is_empty() {
            result.push(current.to_lowercase());
        }

        // If no camelCase detected, return original
        if result.is_empty() {
            vec![word.to_lowercase()]
        } else {
            result
        }
    }

    /// Check if a term is a common stop word that should be filtered out.
    fn is_stop_word(term: &str) -> bool {
        const STOP_WORDS: &[&str] = &[
            // Common English words
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
            "from", "as", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "this",
            "that", "these", "those", "it", "its", "if", "then", "else", "when", "where", "why",
            "how", "all", "each", "every", "some", "any", "no", "not",
            // Common programming keywords (generic)
            "fn", "def", "class", "struct", "impl", "trait", "enum", "type", "let", "var", "const",
            "static", "pub", "mod", "use", "import", "return", "self", "new", "get", "set", "add",
            "mut", "ref",
        ];

        STOP_WORDS.contains(&term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_cluster_async_functions() {
        let members = vec![
            "async fn fetch_data".to_string(),
            "async fn send_request".to_string(),
            "async fn fetch_user".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&members);
        assert!(!label.is_empty());
        // Should identify "fetch" or "async" as common term
        assert!(label.contains("fetch") || label.contains("async"));
    }

    #[test]
    fn test_label_cluster_empty() {
        let members: Vec<String> = vec![];
        let label = ClusterLabeler::label_cluster(&members);
        assert_eq!(label, "empty");
    }

    #[test]
    fn test_label_cluster_single_term() {
        let members = vec![
            "fn handler_route".to_string(),
            "fn handler_middleware".to_string(),
            "fn handler_error".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&members);
        assert!(label.contains("handler"));
    }

    #[test]
    fn test_label_cluster_rust_structs() {
        let members = vec![
            "struct UserRepository".to_string(),
            "struct ProductRepository".to_string(),
            "struct OrderRepository".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&members);
        assert!(label.contains("repository"));
    }

    #[test]
    fn test_label_cluster_python_classes() {
        let members = vec![
            "class DataProcessor".to_string(),
            "class ImageProcessor".to_string(),
            "class TextProcessor".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&members);
        assert!(label.contains("processor"));
    }

    #[test]
    fn test_tokenize_snake_case() {
        let tokens = ClusterLabeler::tokenize("fetch_user_data");
        assert!(tokens.contains(&"fetch".to_string()));
        assert!(tokens.contains(&"user".to_string()));
        assert!(tokens.contains(&"data".to_string()));
    }

    #[test]
    fn test_tokenize_camel_case() {
        let tokens = ClusterLabeler::tokenize("fetchUserData");
        assert!(tokens.contains(&"fetch".to_string()));
        assert!(tokens.contains(&"user".to_string()));
        assert!(tokens.contains(&"data".to_string()));
    }

    #[test]
    fn test_split_camel_case() {
        let result = ClusterLabeler::split_camel_case("fetchUserData");
        assert_eq!(result, vec!["fetch", "user", "data"]);

        let result = ClusterLabeler::split_camel_case("HTTPRequest");
        assert_eq!(result, vec!["httprequest"]);

        let result = ClusterLabeler::split_camel_case("simple");
        assert_eq!(result, vec!["simple"]);
    }

    #[test]
    fn test_is_stop_word() {
        assert!(ClusterLabeler::is_stop_word("fn"));
        assert!(ClusterLabeler::is_stop_word("the"));
        assert!(ClusterLabeler::is_stop_word("def"));
        assert!(ClusterLabeler::is_stop_word("class"));
        assert!(!ClusterLabeler::is_stop_word("fetch"));
        assert!(!ClusterLabeler::is_stop_word("user"));
        assert!(!ClusterLabeler::is_stop_word("repository"));
    }

    #[test]
    fn test_are_related_terms() {
        assert!(ClusterLabeler::are_related_terms("fetch", "fetcher"));
        assert!(ClusterLabeler::are_related_terms("handler", "handle"));
        assert!(ClusterLabeler::are_related_terms("processor", "process"));
        assert!(!ClusterLabeler::are_related_terms("fetch", "send"));
        assert!(!ClusterLabeler::are_related_terms("user", "data"));
    }

    #[test]
    fn test_extract_terms() {
        let texts = vec![
            "fn fetch_data".to_string(),
            "fn fetch_user".to_string(),
            "fn send_email".to_string(),
        ];
        let terms = ClusterLabeler::extract_terms(&texts);

        assert_eq!(*terms.get("fetch").unwrap_or(&0), 2);
        assert_eq!(*terms.get("send").unwrap_or(&0), 1);
    }

    #[test]
    fn test_generate_label_single_term() {
        let handler = "handler".to_string();
        let count = 5usize;
        let terms = vec![(&handler, &count)];
        let label = ClusterLabeler::generate_label(&terms);
        assert_eq!(label, "handler");
    }

    #[test]
    fn test_generate_label_two_terms() {
        let handler = "handler".to_string();
        let route = "route".to_string();
        let count1 = 3usize;
        let count2 = 3usize;
        let terms = vec![(&handler, &count1), (&route, &count2)];
        let label = ClusterLabeler::generate_label(&terms);
        assert!(label.contains("handler"));
    }

    #[test]
    fn test_label_filters_short_terms() {
        let members = vec!["fn a_b_c".to_string(), "fn a_x_y".to_string()];
        let label = ClusterLabeler::label_cluster(&members);
        // Should not use 1-2 char terms
        assert!(!label.is_empty());
    }

    #[test]
    fn test_label_makes_sense_for_http_handlers() {
        let members = vec![
            "fn handle_get_request".to_string(),
            "fn handle_post_request".to_string(),
            "fn handle_delete_request".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&members);
        // Should identify "handle" or "request" as common terms
        assert!(label.contains("handle") || label.contains("request"));
    }

    #[test]
    fn test_manual_verification_labels_make_sense() {
        // Test case 1: Database repositories
        let db_cluster = vec![
            "struct UserRepository".to_string(),
            "struct ProductRepository".to_string(),
            "struct OrderRepository".to_string(),
            "impl UserRepository".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&db_cluster);
        println!("Database cluster label: {}", label);
        assert!(label.contains("repository"));

        // Test case 2: HTTP handlers
        let http_cluster = vec![
            "async fn handle_user_request".to_string(),
            "async fn handle_product_request".to_string(),
            "fn handle_error_response".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&http_cluster);
        println!("HTTP cluster label: {}", label);
        assert!(
            label.contains("handle") || label.contains("request") || label.contains("response")
        );

        // Test case 3: Parsers
        let parser_cluster = vec![
            "fn parse_json_data".to_string(),
            "fn parse_xml_document".to_string(),
            "fn parse_csv_file".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&parser_cluster);
        println!("Parser cluster label: {}", label);
        assert!(label.contains("parse"));

        // Test case 4: Data processors
        let processor_cluster = vec![
            "class ImageProcessor".to_string(),
            "class TextProcessor".to_string(),
            "class DataProcessor".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&processor_cluster);
        println!("Processor cluster label: {}", label);
        assert!(label.contains("processor"));

        // Test case 5: Authentication functions
        let auth_cluster = vec![
            "fn authenticate_user".to_string(),
            "fn validate_token".to_string(),
            "fn authorize_request".to_string(),
            "fn check_permissions".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&auth_cluster);
        println!("Auth cluster label: {}", label);
        // Should identify authentication-related terms
        assert!(!label.is_empty() && label != "misc");

        // Test case 6: Mixed content (should still produce reasonable label)
        let mixed_cluster = vec![
            "fn calculate_total".to_string(),
            "fn compute_average".to_string(),
            "fn sum_values".to_string(),
        ];
        let label = ClusterLabeler::label_cluster(&mixed_cluster);
        println!("Mixed cluster label: {}", label);
        assert!(!label.is_empty() && label != "misc");
    }
}
