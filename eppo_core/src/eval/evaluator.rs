use std::{collections::HashMap, sync::Arc};

use chrono::Utc;

use crate::{
    configuration_store::ConfigurationStore,
    events::AssignmentEvent,
    precomputed::PrecomputedConfiguration,
    ufc::{Assignment, AssignmentValue, VariationType},
    Attributes, Configuration, ContextAttributes, EvaluationError, SdkMetadata, Str,
};

use super::{
    eval_details::{EvaluationDetails, EvaluationResultWithDetails},
    get_assignment, get_assignment_details, get_bandit_action, get_bandit_action_details,
    get_precomputed_configuration, BanditResult,
};

pub struct EvaluatorConfig {
    pub configuration_store: Arc<ConfigurationStore>,
    pub sdk_metadata: SdkMetadata,
}

/// Evaluator simplifies calling into evaluation functions and automatically adds necessary metadata
/// to events (SDK name and version).
pub struct Evaluator {
    config: EvaluatorConfig,
}

impl Evaluator {
    pub fn new(config: EvaluatorConfig) -> Evaluator {
        Evaluator { config }
    }

    pub fn get_assignment(
        &self,
        flag_key: &str,
        subject_key: &Str,
        subject_attributes: &Arc<Attributes>,
        expected_type: Option<VariationType>,
    ) -> Result<Option<Assignment>, EvaluationError> {
        let config = self.get_configuration();
        get_assignment(
            config.as_ref().map(AsRef::as_ref),
            &flag_key,
            &subject_key,
            &subject_attributes,
            expected_type,
            Utc::now(),
        )
    }

    pub fn get_assignment_details(
        &self,
        flag_key: &str,
        subject_key: &Str,
        subject_attributes: &Arc<Attributes>,
        expected_type: Option<VariationType>,
    ) -> (
        EvaluationResultWithDetails<AssignmentValue>,
        Option<AssignmentEvent>,
    ) {
        let config = self.get_configuration();
        get_assignment_details(
            config.as_ref().map(AsRef::as_ref),
            &flag_key,
            &subject_key,
            &subject_attributes,
            expected_type,
            Utc::now(),
        )
    }

    pub fn get_bandit_action(
        &self,
        flag_key: &str,
        subject_key: &Str,
        subject_attributes: &ContextAttributes,
        actions: &HashMap<Str, ContextAttributes>,
        default_variation: &Str,
    ) -> BanditResult {
        let configuration = self.get_configuration();
        get_bandit_action(
            configuration.as_ref().map(|it| it.as_ref()),
            flag_key,
            subject_key,
            subject_attributes,
            actions,
            default_variation,
            Utc::now(),
            &self.config.sdk_metadata,
        )
    }

    pub fn get_bandit_action_details(
        &self,
        flag_key: &str,
        subject_key: &Str,
        subject_attributes: &ContextAttributes,
        actions: &HashMap<Str, ContextAttributes>,
        default_variation: &Str,
    ) -> (BanditResult, EvaluationDetails) {
        let configuration = self.get_configuration();
        get_bandit_action_details(
            configuration.as_ref().map(|it| it.as_ref()),
            flag_key,
            subject_key,
            subject_attributes,
            actions,
            default_variation,
            Utc::now(),
            &self.config.sdk_metadata,
        )
    }

    pub fn get_precomputed_configuration(
        &self,
        subject_key: &Str,
        subject_attributes: &Arc<ContextAttributes>,
        flag_actions: &HashMap<
            /* flag_key: */ Str,
            HashMap</* action_key: */ Str, ContextAttributes>,
        >,
    ) -> PrecomputedConfiguration {
        let configuration = self.get_configuration();
        get_precomputed_configuration(
            configuration.as_ref().map(AsRef::as_ref),
            subject_key,
            subject_attributes,
            flag_actions,
            Utc::now(),
        )
    }

    fn get_configuration(&self) -> Option<Arc<Configuration>> {
        self.config.configuration_store.get_configuration()
    }
}
