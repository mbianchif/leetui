use std::{env, fs, io, process::Command};

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
    let description_path = dir_path.join("README.md");
    let md = html2md::parse_html(&question.content);
    fs::write(description_path, md)?;

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

pub fn open_editor(app: &App) -> io::Result<()> {
    let Some(ref question) = app.question else {
        unreachable!();
    };

    let slug = &question.title_slug;
    let index = app.file_list_state.selected().unwrap_or_default();
    let file_name = &app.local_files[index];
    let dir_path = env::home_dir()
        .unwrap_or_default()
        .join(".leetui")
        .join(slug);

    let description_path = dir_path.join("README.md");
    let file_path = dir_path.join(file_name);

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut cmd = Command::new(editor)
        .arg(description_path)
        .arg(file_path)
        .spawn()?;

    cmd.wait()?;
    Ok(())
}
