use std::path::PathBuf;

/// A turn input entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserInput {
    Text { text: String },
    LocalImage { path: PathBuf },
}

impl UserInput {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn local_image(path: impl Into<PathBuf>) -> Self {
        Self::LocalImage { path: path.into() }
    }
}

/// Input accepted by `Thread::run` and `Thread::run_streamed`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Input {
    Text(String),
    Items(Vec<UserInput>),
}

impl From<&str> for Input {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for Input {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<Vec<UserInput>> for Input {
    fn from(value: Vec<UserInput>) -> Self {
        Self::Items(value)
    }
}

impl<const N: usize> From<[UserInput; N]> for Input {
    fn from(value: [UserInput; N]) -> Self {
        Self::Items(value.into())
    }
}

pub(crate) fn normalize_input(input: Input) -> (String, Vec<PathBuf>) {
    match input {
        Input::Text(prompt) => (prompt, Vec::new()),
        Input::Items(items) => {
            let mut prompt_parts = Vec::new();
            let mut images = Vec::new();
            for item in items {
                match item {
                    UserInput::Text { text } => prompt_parts.push(text),
                    UserInput::LocalImage { path } => images.push(path),
                }
            }
            (prompt_parts.join("\n\n"), images)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_input_combines_text_and_collects_images() {
        let (prompt, images) = normalize_input(Input::Items(vec![
            UserInput::text("Describe file changes"),
            UserInput::local_image("ui.png"),
            UserInput::text("Focus on impacted tests"),
        ]));

        assert_eq!(prompt, "Describe file changes\n\nFocus on impacted tests");
        assert_eq!(images, vec![PathBuf::from("ui.png")]);
    }
}
