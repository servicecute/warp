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
