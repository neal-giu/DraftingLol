#[derive(Debug, Clone, Default)]
pub struct LiveService;

impl LiveService {
    #[must_use]
    pub const fn status_label(&self) -> &'static str {
        "not_started"
    }
}
