//! Modal overlay system for CRUD forms and confirmation dialogs.
//! Detail views are now full-page in the content area (see nav::Screen).

#[derive(Debug)]
pub enum Modal {
    /// Text input form with named fields.
    Form(FormState),
    /// Confirmation dialog (e.g. delete).
    Confirm {
        title: String,
        message: String,
        on_confirm: ConfirmAction,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ConfirmAction {
    DeleteNote(String),
    DeleteAdr(String),
    UpdateJobStatus { job_id: String, new_status: String },
}

#[derive(Debug)]
pub struct FormState {
    pub title: String,
    pub fields: Vec<FormField>,
    pub focused_field: usize,
    pub on_submit: FormAction,
}

#[derive(Debug)]
pub struct FormField {
    pub label: String,
    pub value: String,
    #[allow(dead_code)]
    pub multiline: bool,
}

#[derive(Debug, Clone)]
pub enum FormAction {
    CreateNote,
    EditNote(String),
    CreateAdr,
    EditUserPrefs,
}

impl FormState {
    pub fn new_note() -> Self {
        Self {
            title: "New Note".to_string(),
            fields: vec![
                FormField {
                    label: "Title".to_string(),
                    value: String::new(),
                    multiline: false,
                },
                FormField {
                    label: "Content".to_string(),
                    value: String::new(),
                    multiline: true,
                },
            ],
            focused_field: 0,
            on_submit: FormAction::CreateNote,
        }
    }

    pub fn edit_note(id: &str, title: &str, content: &str) -> Self {
        Self {
            title: "Edit Note".to_string(),
            fields: vec![
                FormField {
                    label: "Title".to_string(),
                    value: title.to_string(),
                    multiline: false,
                },
                FormField {
                    label: "Content".to_string(),
                    value: content.to_string(),
                    multiline: true,
                },
            ],
            focused_field: 0,
            on_submit: FormAction::EditNote(id.to_string()),
        }
    }

    pub fn new_adr() -> Self {
        Self {
            title: "New ADR".to_string(),
            fields: vec![
                FormField {
                    label: "Title".to_string(),
                    value: String::new(),
                    multiline: false,
                },
                FormField {
                    label: "Context".to_string(),
                    value: String::new(),
                    multiline: true,
                },
                FormField {
                    label: "Decision".to_string(),
                    value: String::new(),
                    multiline: true,
                },
            ],
            focused_field: 0,
            on_submit: FormAction::CreateAdr,
        }
    }

    pub fn edit_user_prefs(prefs: &[(String, String)]) -> Self {
        use super::data::USER_PREF_KEYS;
        let fields = USER_PREF_KEYS
            .iter()
            .enumerate()
            .map(|(i, (_key, label))| {
                let value = prefs.get(i).map(|(_, v)| v.clone()).unwrap_or_default();
                FormField {
                    label: label.to_string(),
                    value,
                    multiline: false,
                }
            })
            .collect();
        Self {
            title: "Edit User Preferences".to_string(),
            fields,
            focused_field: 0,
            on_submit: FormAction::EditUserPrefs,
        }
    }

    pub fn next_field(&mut self) {
        if self.focused_field < self.fields.len().saturating_sub(1) {
            self.focused_field += 1;
        }
    }

    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.saturating_sub(1);
    }

    pub fn type_char(&mut self, c: char) {
        self.fields[self.focused_field].value.push(c);
    }

    pub fn backspace(&mut self) {
        self.fields[self.focused_field].value.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_field_navigation() {
        let mut form = FormState::new_adr();
        assert_eq!(form.focused_field, 0);
        form.next_field();
        assert_eq!(form.focused_field, 1);
        form.next_field();
        assert_eq!(form.focused_field, 2);
        form.next_field(); // clamps
        assert_eq!(form.focused_field, 2);
        form.prev_field();
        assert_eq!(form.focused_field, 1);
    }

    #[test]
    fn form_typing() {
        let mut form = FormState::new_note();
        form.type_char('H');
        form.type_char('i');
        assert_eq!(form.fields[0].value, "Hi");
        form.backspace();
        assert_eq!(form.fields[0].value, "H");
    }
}
