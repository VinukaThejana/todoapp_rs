#[derive(Debug, Default)]
pub struct TokenParams {
    pub ajti: Option<String>,
    pub rjti: Option<String>,
}

impl TokenParams {
    pub fn with_ajti(mut self, ajti: String) -> Self {
        self.ajti = Some(ajti);
        self
    }

    pub fn with_rjti(mut self, rjti: String) -> Self {
        self.rjti = Some(rjti);
        self
    }
}
