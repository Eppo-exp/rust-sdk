#![allow(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{timestamp::Timestamp, Str};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditResponse {
    pub bandits: HashMap<Str, BanditConfiguration>,
    pub updated_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditConfiguration {
    pub bandit_key: String,
    pub model_name: String,
    pub model_version: Str,
    pub model_data: BanditModelData,
    pub updated_at: Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditModelData {
    pub gamma: f64,
    pub default_action_score: f64,
    pub action_probability_floor: f64,
    pub coefficients: HashMap<String, BanditCoefficients>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditCoefficients {
    pub action_key: String,
    pub intercept: f64,
    pub subject_numeric_coefficients: Vec<BanditNumericAttributeCoefficient>,
    pub subject_categorical_coefficients: Vec<BanditCategoricalAttributeCoefficient>,
    pub action_numeric_coefficients: Vec<BanditNumericAttributeCoefficient>,
    pub action_categorical_coefficients: Vec<BanditCategoricalAttributeCoefficient>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditNumericAttributeCoefficient {
    pub attribute_key: String,
    pub coefficient: f64,
    pub missing_value_coefficient: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BanditCategoricalAttributeCoefficient {
    pub attribute_key: String,
    pub value_coefficients: HashMap<String, f64>,
    pub missing_value_coefficient: f64,
}
