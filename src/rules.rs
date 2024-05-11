use derive_more::From;
use regex::Regex;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{client::AttributeValue, ufc::Value, SubjectAttributes};

#[derive(Debug, Serialize, Deserialize, From)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    conditions: Vec<Condition>,
}

impl Rule {
    pub fn eval(&self, attributes: &SubjectAttributes) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.eval(attributes))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    operator: Operator,
    attribute: String,
    value: ConditionValue,
}

impl Condition {
    pub fn eval(&self, attributes: &SubjectAttributes) -> bool {
        self.operator
            .eval(attributes.get(&self.attribute), &self.value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    Multiple(Vec<Value>),
    Single(Value),
}

impl<T: Into<Value>> From<T> for ConditionValue {
    fn from(value: T) -> Self {
        Self::Single(value.into())
    }
}
impl<T: Into<Value>> From<Vec<T>> for ConditionValue {
    fn from(value: Vec<T>) -> Self {
        Self::Multiple(value.into_iter().map(Into::into).collect())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Operator {
    Matches,
    NotMatches,
    Gte,
    Gt,
    Lte,
    Lt,
    OneOf,
    NotOneOf,
    IsNull,
}

impl Operator {
    /// Applying `Operator` to the values. Returns `false` if the operator cannot be applied or
    /// there's a misconfiguration.
    pub fn eval(
        &self,
        attribute: Option<&AttributeValue>,
        condition_value: &ConditionValue,
    ) -> bool {
        self.try_eval(attribute, condition_value).unwrap_or(false)
    }

    /// Try applying `Operator` to the values, returning `None` if the operator cannot be applied.
    fn try_eval(
        &self,
        attribute: Option<&AttributeValue>,
        condition_value: &ConditionValue,
    ) -> Option<bool> {
        match self {
            Self::Matches | Self::NotMatches => {
                let s = match attribute {
                    Some(AttributeValue::String(s)) => s,
                    _ => return None,
                };
                let regex = match condition_value {
                    ConditionValue::Single(Value::String(s)) => Regex::new(s).ok()?,
                    _ => return None,
                };
                let matches = regex.is_match(s);
                Some(if matches!(self, Self::Matches) {
                    matches
                } else {
                    !matches
                })
            }

            Self::OneOf | Self::NotOneOf => {
                let s = match attribute {
                    Some(AttributeValue::String(s)) => s.clone(),
                    Some(AttributeValue::Number(n)) => n.to_string(),
                    Some(AttributeValue::Boolean(b)) => b.to_string(),
                    _ => return None,
                };
                let values = match condition_value {
                    ConditionValue::Multiple(v) => v,
                    _ => return None,
                };
                let is_one_of = values.iter().any(|v| {
                    if let Value::String(v) = v {
                        v == &s
                    } else {
                        false
                    }
                });
                Some(if *self == Self::OneOf {
                    is_one_of
                } else {
                    !is_one_of
                })
            }

            Self::IsNull => {
                let is_null =
                    attribute.is_none() || attribute.is_some_and(|v| v == &AttributeValue::Null);
                match condition_value {
                    ConditionValue::Single(Value::Boolean(true)) => Some(is_null),
                    ConditionValue::Single(Value::Boolean(false)) => Some(!is_null),
                    _ => None,
                }
            }

            Self::Gte | Self::Gt | Self::Lte | Self::Lt => {
                let condition_version = match condition_value {
                    ConditionValue::Single(Value::String(s)) => Version::parse(s).ok(),
                    _ => None,
                };

                if let Some(condition_version) = condition_version {
                    // semver comparison

                    let attribute_version = match attribute {
                        Some(AttributeValue::String(s)) => Version::parse(s).ok(),
                        _ => None,
                    }?;

                    Some(match self {
                        Self::Gt => attribute_version > condition_version,
                        Self::Gte => attribute_version >= condition_version,
                        Self::Lt => attribute_version < condition_version,
                        Self::Lte => attribute_version <= condition_version,
                        _ => {
                            // unreachable
                            return None;
                        }
                    })
                } else {
                    // numeric comparison
                    let condition_value = match condition_value {
                        ConditionValue::Single(Value::Number(n)) => *n,
                        ConditionValue::Single(Value::String(s)) => s.parse().ok()?,
                        _ => return None,
                    };

                    let attribute_value = match attribute {
                        Some(AttributeValue::Number(n)) => *n,
                        Some(AttributeValue::String(s)) => s.parse().ok()?,
                        _ => return None,
                    };

                    Some(match self {
                        Self::Gt => attribute_value > condition_value,
                        Self::Gte => attribute_value >= condition_value,
                        Self::Lt => attribute_value < condition_value,
                        Self::Lte => attribute_value <= condition_value,
                        _ => {
                            // unreachable
                            return None;
                        }
                    })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        rules::{Condition, Operator},
        ufc::Value,
    };

    use super::Rule;

    #[test]
    fn matches_regex() {
        assert!(Operator::Matches.eval(Some(&"test@example.com".into()), &"^test.*".into()));
        assert!(!Operator::Matches.eval(Some(&"example@test.com".into()), &"^test.*".into()));
    }

    #[test]
    fn not_matches_regex() {
        assert!(!Operator::NotMatches.eval(Some(&"test@example.com".into()), &"^test.*".into()));
        assert!(!Operator::NotMatches.eval(None, &"^test.*".into()));
        assert!(Operator::NotMatches.eval(Some(&"example@test.com".into()), &"^test.*".into()));
    }

    #[test]
    fn one_of() {
        assert!(Operator::OneOf.eval(
            Some(&"alice".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));
        assert!(Operator::OneOf.eval(
            Some(&"bob".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));
        assert!(!Operator::OneOf.eval(
            Some(&"charlie".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));
    }

    #[test]
    fn not_one_of() {
        assert!(!Operator::NotOneOf.eval(
            Some(&"alice".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));
        assert!(!Operator::NotOneOf.eval(
            Some(&"bob".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));
        assert!(Operator::NotOneOf.eval(
            Some(&"charlie".into()),
            &vec![Value::from("alice"), Value::from("bob")].into()
        ));

        // NOT_ONE_OF fails when attribute is not specified
        assert!(
            !Operator::NotOneOf.eval(None, &vec![Value::from("alice"), Value::from("bob")].into())
        );
    }

    #[test]
    fn one_of_int() {
        assert!(Operator::OneOf.eval(Some(&42.0.into()), &vec![Value::from("42")].into()));
    }

    #[test]
    fn one_of_bool() {
        assert!(Operator::OneOf.eval(Some(&true.into()), &vec![Value::from("true")].into()));
        assert!(Operator::OneOf.eval(Some(&false.into()), &vec![Value::from("false")].into()));
        assert!(!Operator::OneOf.eval(Some(&1.0.into()), &vec![Value::from("true")].into()));
        assert!(!Operator::OneOf.eval(Some(&0.0.into()), &vec![Value::from("false")].into()));
        assert!(!Operator::OneOf.eval(None, &vec![Value::from("true")].into()));
        assert!(!Operator::OneOf.eval(None, &vec![Value::from("false")].into()));
    }

    #[test]
    fn is_null() {
        assert!(Operator::IsNull.eval(None, &true.into()));
        assert!(!Operator::IsNull.eval(Some(&10.0.into()), &true.into()));
    }

    #[test]
    fn is_not_null() {
        assert!(!Operator::IsNull.eval(None, &false.into()));
        assert!(Operator::IsNull.eval(Some(&10.0.into()), &false.into()));
    }

    #[test]
    fn gte() {
        assert!(Operator::Gte.eval(Some(&18.0.into()), &18.0.into()));
        assert!(!Operator::Gte.eval(Some(&17.0.into()), &18.0.into()));
    }
    #[test]
    fn gt() {
        assert!(Operator::Gt.eval(Some(&19.0.into()), &18.0.into()));
        assert!(!Operator::Gt.eval(Some(&18.0.into()), &18.0.into()));
    }
    #[test]
    fn lte() {
        assert!(Operator::Lte.eval(Some(&18.0.into()), &18.0.into()));
        assert!(!Operator::Lte.eval(Some(&19.0.into()), &18.0.into()));
    }
    #[test]
    fn lt() {
        assert!(Operator::Lt.eval(Some(&17.0.into()), &18.0.into()));
        assert!(!Operator::Lt.eval(Some(&18.0.into()), &18.0.into()));
    }

    #[test]
    fn semver_gte() {
        assert!(Operator::Gte.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(Operator::Gte.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(!Operator::Gte.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(Operator::Gte.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(!Operator::Gte.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_gt() {
        assert!(Operator::Gt.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(!Operator::Gt.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(!Operator::Gt.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(Operator::Gt.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(!Operator::Gt.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_lte() {
        assert!(!Operator::Lte.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(Operator::Lte.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(Operator::Lte.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(!Operator::Lte.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(Operator::Lte.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_lt() {
        assert!(!Operator::Lt.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(!Operator::Lt.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(Operator::Lt.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(!Operator::Lt.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(Operator::Lt.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }

    #[test]
    fn empty_rule() {
        let rule = Rule { conditions: vec![] };
        assert!(rule.eval(&HashMap::from([])));
    }

    #[test]
    fn single_condition_rule() {
        let rule = Rule {
            conditions: vec![Condition {
                attribute: "age".into(),
                operator: Operator::Gt,
                value: 10.0.into(),
            }],
        };
        assert!(rule.eval(&HashMap::from([("age".into(), 11.0.into())])));
    }

    #[test]
    fn two_condition_rule() {
        let rule = Rule {
            conditions: vec![
                Condition {
                    attribute: "age".into(),
                    operator: Operator::Gt,
                    value: 18.0.into(),
                },
                Condition {
                    attribute: "age".into(),
                    operator: Operator::Lt,
                    value: 100.0.into(),
                },
            ],
        };
        assert!(rule.eval(&HashMap::from([("age".into(), 20.0.into())])));
        assert!(!rule.eval(&HashMap::from([("age".into(), 17.0.into())])));
        assert!(!rule.eval(&HashMap::from([("age".into(), 110.0.into())])));
    }

    #[test]
    fn missing_attribute() {
        let rule = Rule {
            conditions: vec![Condition {
                attribute: "age".into(),
                operator: Operator::Gt,
                value: 10.0.into(),
            }],
        };
        assert!(!rule.eval(&HashMap::from([("name".into(), "alice".into())])));
    }
}
