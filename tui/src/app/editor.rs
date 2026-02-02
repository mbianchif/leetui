use std::{env, fs, io};

use crate::app::App;

pub fn create_file(app: &App) -> io::Result<()> {
    let Some(ref question) = app.question else {
        unreachable!();
    };

    let slug = &question.title_slug;
    let dir_path = env::home_dir()
        .unwrap_or_default()
        .join(".leetui")
        .join(slug);

    fs::create_dir_all(&dir_path)?;
    let file_name = &app.new_file_input;
    let file_path = dir_path.join(file_name);
    let lang = app.detected_language.as_ref().unwrap();

    let snippet: &str = question
        .code_snippets
        .iter()
        .find(|s| s.lang == *lang)
        .map(|s| s.code.as_ref())
        .unwrap_or_default();

    fs::write(file_path, snippet.as_bytes())
}
