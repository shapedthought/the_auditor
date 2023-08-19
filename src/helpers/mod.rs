use dialoguer::{Select, theme::ColorfulTheme, console::Term};

pub fn select_selection(selections: &[&str], prompt: String) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&prompt)
        .default(0)
        .items(selections)
        .interact_on_opt(&Term::stderr())
        .unwrap()
        .unwrap();
    selection
}