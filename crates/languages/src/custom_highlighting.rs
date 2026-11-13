use std::ops::Range;

use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CustomHighlightRuleDefinition {
    pub pattern: String,
    pub scope: String,
    #[serde(default)]
    pub capture: usize,
}

#[derive(Clone, Debug)]
struct CustomHighlightRule {
    pattern: Regex,
    scope: String,
    capture: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CustomHighlighting {
    rules: Vec<CustomHighlightRule>,
}

#[derive(Clone, Debug)]
pub struct CustomHighlightMatch {
    pub range: Range<usize>,
    pub scope: String,
}

impl CustomHighlighting {
    pub fn from_definitions(definitions: &[CustomHighlightRuleDefinition]) -> Result<Self, String> {
        let rules = definitions
            .iter()
            .map(Self::compile_rule)
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self { rules })
    }

    /// Compile user-provided rules without allowing one malformed rule to hide
    /// the built-in rules. Built-in rules are still loaded strictly by the
    /// language registry, while user rules are intentionally best-effort.
    pub fn from_definitions_lossy(
        definitions: &[CustomHighlightRuleDefinition],
    ) -> (Self, Vec<String>) {
        let mut rules = Vec::with_capacity(definitions.len());
        let mut errors = Vec::new();

        for definition in definitions {
            match Self::compile_rule(definition) {
                Ok(rule) => rules.push(rule),
                Err(error) => errors.push(error),
            }
        }

        (Self { rules }, errors)
    }

    pub fn extend(&mut self, other: Self) {
        self.rules.extend(other.rules);
    }

    fn compile_rule(
        definition: &CustomHighlightRuleDefinition,
    ) -> Result<CustomHighlightRule, String> {
        let pattern = Regex::new(&definition.pattern).map_err(|error| {
            format!(
                "invalid custom highlighting regex {:?}: {error}",
                definition.pattern
            )
        })?;
        if definition.capture > pattern.captures_len().saturating_sub(1) {
            return Err(format!(
                "custom highlighting regex {:?} has no capture group {}",
                definition.pattern, definition.capture
            ));
        }
        Ok(CustomHighlightRule {
            pattern,
            scope: definition.scope.clone(),
            capture: definition.capture,
        })
    }

    pub fn matches(&self, text: &str) -> Vec<CustomHighlightMatch> {
        self.rules
            .iter()
            .flat_map(|rule| {
                rule.pattern.captures_iter(text).filter_map(|captures| {
                    let range = captures.get(rule.capture).map(|capture| capture.range())?;
                    (!range.is_empty()).then(|| CustomHighlightMatch {
                        range,
                        scope: rule.scope.clone(),
                    })
                })
            })
            .collect()
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct UserLanguageConfig {
    #[serde(default)]
    pub languages: Vec<UserLanguageDefinition>,
}

#[derive(Debug, Default, Deserialize)]
pub struct UserLanguageDefinition {
    pub language: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub colors: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub custom_highlights: Vec<CustomHighlightRuleDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_rules_return_captured_ranges_and_scopes() {
        let highlighting = CustomHighlighting::from_definitions(&[
            CustomHighlightRuleDefinition {
                pattern: r"@(codegen|firestore)".into(),
                scope: "annotation.marker.gleam".into(),
                capture: 0,
            },
            CustomHighlightRuleDefinition {
                pattern: r"@codegen\s+([a-z]+)".into(),
                scope: "annotation.namespace.gleam".into(),
                capture: 1,
            },
        ])
        .expect("test rules should compile");

        let text = "/// @codegen json @firestore";
        let matches = highlighting.matches(text);

        assert_eq!(matches.len(), 3);
        let matched_text: Vec<_> = matches
            .iter()
            .map(|matched| &text[matched.range.clone()])
            .collect();
        assert!(matched_text.contains(&"@codegen"));
        assert!(matched_text.contains(&"@firestore"));
        assert!(matched_text.contains(&"json"));
        assert!(
            matches
                .iter()
                .any(|matched| matched.scope == "annotation.marker.gleam")
        );
        assert!(
            matches
                .iter()
                .any(|matched| matched.scope == "annotation.namespace.gleam")
        );
    }

    #[test]
    fn invalid_capture_group_is_rejected() {
        let result = CustomHighlighting::from_definitions(&[CustomHighlightRuleDefinition {
            pattern: "@codegen".into(),
            scope: "annotation.marker.gleam".into(),
            capture: 1,
        }]);

        assert!(result.is_err());
    }

    #[test]
    fn lossy_rules_keep_valid_rules_and_report_invalid_rules() {
        let (highlighting, errors) = CustomHighlighting::from_definitions_lossy(&[
            CustomHighlightRuleDefinition {
                pattern: r"@codegen".into(),
                scope: "annotation.marker.gleam".into(),
                capture: 0,
            },
            CustomHighlightRuleDefinition {
                pattern: "@broken(".into(),
                scope: "annotation.invalid.gleam".into(),
                capture: 0,
            },
        ]);

        assert_eq!(errors.len(), 1);
        assert_eq!(highlighting.matches("/// @codegen").len(), 1);
    }
}
