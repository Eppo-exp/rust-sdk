pub struct EppoClient<'a> {
    config: EppoClientConfig,
}

impl<'a> EppoClient<'a> {
    pub fn new(config: EppoClientConfig) -> EppoClient;

    pub fn get_assignment(&self, flag_key: &str, subject_key: &str, subject_attributes: &SubjectAttributes) -> Option<Assignment>;

    // block till we get first configuration?
    pub fn start_poller_thread(&mut self) -> ???; // Return something that allows to wait for initialization.
}

pub struct EppoClientConfig<'a> {
    api_key: String,
    base_url: String,
    assignment_logger: Box<dyn AssignmentLogger + Send + Sync + 'a>;
}
impl<'a> EppoClientConfig<'a> {
    pub fn new<S: Into<String>>(api_key: S) -> Self;
    pub fn base_url<S: Into<String>>(&mut self, base_url: S) -> &mut Self;
    pub fn assignment_logger(&mut self, assignment_logger: impl AssignmentLogger + Send + Sync + 'a) -> &mut Self;
    pub fn to_client(self) -> EppoClient<'a>;
}

trait AssignmentLogger {
    fn log_assignment(&self, event: AssignmentEvent);
}

pub struct AssignmentEvent {
    pub experiment: String,
    pub subject: String,
    pub variation: String,
    pub timestamp: String,
    pub subject_attributes: SubjectAttributes,
    pub feature_flag: String,
    pub allocation: String,
}

pub enum AssignmentValue {
    String(String),
    Integer(i64),
    Numeric(Number),
    Boolean(bool),
    Json(serde_json::Value),
}

impl AssignmentValue {
    pub fn is_string(&self) -> bool;
    pub fn as_str(&self) -> Option<&str>;

    pub fn is_boolean(&self) -> bool;
    pub fn as_bool(&self) -> Option<bool>;

    pub fn is_i64(&self) -> bool;
    pub fn as_i64(&self) -> Option<i64>;

    pub fn is_f64(&self) -> bool;
    pub fn as_f64(&self) -> Option<f64>;

    pub fn is_json(&self) -> bool;
    pub fn as_json(&self) -> Option<serde_json::Value>;
}
