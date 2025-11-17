use crate::config::Config;
use crate::requests::{api_delete, api_post, api_put};
use serde_json::Value;

pub enum CurrentScreen {
    Main,
    Editing,
    Adding,
    Deleting,
}

pub struct CurrentlyEditing {
    pub key: String,
    pub value: String,
    pub changed: bool,
}

pub struct CurrentlyAdding {
    pub value: String,
}

pub struct CurrentlyDeleting {
    pub key: String,
    pub are_you_sure: bool, //Pretty sure. Threw a trash bag into space at work.
}

pub struct App {
    pub json: Value,
    pub config: Config,
    pub key_path: Vec<String>,
    pub locations: Vec<usize>,
    pub current_screen: CurrentScreen,
    pub currently_editing: Option<CurrentlyEditing>,
    pub currently_adding: Option<CurrentlyAdding>,
    pub currently_deleting: Option<CurrentlyDeleting>,
}

impl App {
    pub fn new(json: Value, config: Config) -> App {
        App {
            json,
            config,
            key_path: vec![],
            locations: vec![],
            current_screen: CurrentScreen::Main,
            currently_editing: None,
            currently_adding: None,
            currently_deleting: None,
        }
    }

    pub fn save_edited_value(&self) -> color_eyre::Result<()> {
        if let Some(editing) = &self.currently_editing {
            let mut key_path = self.key_path.clone();
            key_path.push(editing.key.clone());
            let path = key_path
                .iter()
                .map(|v| {
                    format!(
                        "/{}",
                        match v.trim_start_matches('0') {
                            "" => "0",
                            s => s,
                        }
                    )
                })
                .collect::<String>();
            api_put(&path, &editing.value, &self.config.api_key)?;
        }
        Ok(())
    }

    pub fn push_object_to_array(&self) -> color_eyre::Result<()> {
        if let Some(adding) = &self.currently_adding {
            let key_path = self.key_path.clone();
            let path = key_path
                .iter()
                .map(|v| {
                    format!(
                        "/{}",
                        match v.trim_start_matches('0') {
                            "" => "0",
                            s => s,
                        }
                    )
                })
                .collect::<String>();
            api_post(&path, &adding.value, &self.config.api_key)?;
        }
        Ok(())
    }

    pub fn delete_value(&self) -> color_eyre::Result<()> {
        if let Some(deleting) = &self.currently_deleting {
            let mut key_path = self.key_path.clone();
            key_path.push(deleting.key.clone());
            let path = key_path
                .iter()
                .map(|v| {
                    format!(
                        "/{}",
                        match v.trim_start_matches('0') {
                            "" => "0",
                            s => s,
                        }
                    )
                })
                .collect::<String>();
            api_delete(&path, &self.config.api_key)?;
        }
        Ok(())
    }
}
