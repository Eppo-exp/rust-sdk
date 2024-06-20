use regex::Regex;
use semver::Version;

use crate::{
    ufc::{Condition, ConditionOperator, ConditionValue, Rule, Value},
    AttributeValue, Attributes,
};

impl Rule {
    pub(crate) fn eval(&self, attributes: &Attributes) -> bool {
        self.conditions
            .iter()
            .all(|condition| condition.eval(attributes))
    }
}

impl Condition {
    fn eval(&self, attributes: &Attributes) -> bool {
        self.operator
            .eval(attributes.get(&self.attribute), &self.value)
    }
}

impl ConditionOperator {
    /// Applying `Operator` to the values. Returns `false` if the operator cannot be applied or
    /// there's a misconfiguration.
    fn eval(&self, attribute: Option<&AttributeValue>, condition_value: &ConditionValue) -> bool {
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
                let is_one_of = values.iter().any(|v| v == &s);
                let has_to_be_one_of = *self == Self::OneOf;
                Some(is_one_of == has_to_be_one_of)
            }

            Self::IsNull => {
                let is_null = attribute.is_none() || attribute == Some(&AttributeValue::Null);
                let ConditionValue::Single(Value::Boolean(expected_null)) = condition_value else {
                    return None;
                };
                Some(is_null == *expected_null)
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

    use crate::ufc::{Condition, ConditionOperator, Rule};

    #[test]
    fn matches_regex() {
        assert!(
            ConditionOperator::Matches.eval(Some(&"test@example.com".into()), &"^test.*".into())
        );
        assert!(
            !ConditionOperator::Matches.eval(Some(&"example@test.com".into()), &"^test.*".into())
        );
    }

    #[test]
    fn not_matches_regex() {
        assert!(!ConditionOperator::NotMatches
            .eval(Some(&"test@example.com".into()), &"^test.*".into()));
        assert!(!ConditionOperator::NotMatches.eval(None, &"^test.*".into()));
        assert!(
            ConditionOperator::NotMatches.eval(Some(&"example@test.com".into()), &"^test.*".into())
        );
    }

    #[test]
    fn one_of() {
        assert!(ConditionOperator::OneOf.eval(
            Some(&"alice".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));
        assert!(ConditionOperator::OneOf.eval(
            Some(&"bob".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));
        assert!(!ConditionOperator::OneOf.eval(
            Some(&"charlie".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));
    }

    #[test]
    fn not_one_of() {
        assert!(!ConditionOperator::NotOneOf.eval(
            Some(&"alice".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));
        assert!(!ConditionOperator::NotOneOf.eval(
            Some(&"bob".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));
        assert!(ConditionOperator::NotOneOf.eval(
            Some(&"charlie".into()),
            &vec![String::from("alice"), String::from("bob")].into()
        ));

        // NOT_ONE_OF fails when attribute is not specified
        assert!(!ConditionOperator::NotOneOf.eval(
            None,
            &vec![String::from("alice"), String::from("bob")].into()
        ));
    }

    #[test]
    fn one_of_int() {
        assert!(ConditionOperator::OneOf.eval(Some(&42.0.into()), &vec![String::from("42")].into()));
    }

    #[test]
    fn one_of_bool() {
        assert!(
            ConditionOperator::OneOf.eval(Some(&true.into()), &vec![String::from("true")].into())
        );
        assert!(
            ConditionOperator::OneOf.eval(Some(&false.into()), &vec![String::from("false")].into())
        );
        assert!(
            !ConditionOperator::OneOf.eval(Some(&1.0.into()), &vec![String::from("true")].into())
        );
        assert!(
            !ConditionOperator::OneOf.eval(Some(&0.0.into()), &vec![String::from("false")].into())
        );
        assert!(!ConditionOperator::OneOf.eval(None, &vec![String::from("true")].into()));
        assert!(!ConditionOperator::OneOf.eval(None, &vec![String::from("false")].into()));
    }

    #[test]
    fn is_null() {
        assert!(ConditionOperator::IsNull.eval(None, &true.into()));
        assert!(!ConditionOperator::IsNull.eval(Some(&10.0.into()), &true.into()));
    }

    #[test]
    fn is_not_null() {
        assert!(!ConditionOperator::IsNull.eval(None, &false.into()));
        assert!(ConditionOperator::IsNull.eval(Some(&10.0.into()), &false.into()));
    }

    #[test]
    fn gte() {
        assert!(ConditionOperator::Gte.eval(Some(&18.0.into()), &18.0.into()));
        assert!(!ConditionOperator::Gte.eval(Some(&17.0.into()), &18.0.into()));
    }
    #[test]
    fn gt() {
        assert!(ConditionOperator::Gt.eval(Some(&19.0.into()), &18.0.into()));
        assert!(!ConditionOperator::Gt.eval(Some(&18.0.into()), &18.0.into()));
    }
    #[test]
    fn lte() {
        assert!(ConditionOperator::Lte.eval(Some(&18.0.into()), &18.0.into()));
        assert!(!ConditionOperator::Lte.eval(Some(&19.0.into()), &18.0.into()));
    }
    #[test]
    fn lt() {
        assert!(ConditionOperator::Lt.eval(Some(&17.0.into()), &18.0.into()));
        assert!(!ConditionOperator::Lt.eval(Some(&18.0.into()), &18.0.into()));
    }

    #[test]
    fn semver_gte() {
        assert!(ConditionOperator::Gte.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(ConditionOperator::Gte.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(!ConditionOperator::Gte.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(ConditionOperator::Gte.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(!ConditionOperator::Gte.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_gt() {
        assert!(ConditionOperator::Gt.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(!ConditionOperator::Gt.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(!ConditionOperator::Gt.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(ConditionOperator::Gt.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(!ConditionOperator::Gt.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_lte() {
        assert!(!ConditionOperator::Lte.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(ConditionOperator::Lte.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(ConditionOperator::Lte.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(!ConditionOperator::Lte.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(ConditionOperator::Lte.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
    }
    #[test]
    fn semver_lt() {
        assert!(!ConditionOperator::Lt.eval(Some(&"1.0.1".into()), &"1.0.0".into()));
        assert!(!ConditionOperator::Lt.eval(Some(&"1.0.0".into()), &"1.0.0".into()));
        assert!(ConditionOperator::Lt.eval(Some(&"1.2.0".into()), &"1.10.0".into()));
        assert!(!ConditionOperator::Lt.eval(Some(&"1.13.0".into()), &"1.5.0".into()));
        assert!(ConditionOperator::Lt.eval(Some(&"0.9.9".into()), &"1.0.0".into()));
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
                operator: ConditionOperator::Gt,
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
                    operator: ConditionOperator::Gt,
                    value: 18.0.into(),
                },
                Condition {
                    attribute: "age".into(),
                    operator: ConditionOperator::Lt,
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
                operator: ConditionOperator::Gt,
                value: 10.0.into(),
            }],
        };
        assert!(!rule.eval(&HashMap::from([("name".into(), "alice".into())])));
    }
}
