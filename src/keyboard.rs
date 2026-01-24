use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt;

/// Button action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ButtonAction {
    /// Text button
    #[serde(rename = "text")]
    Text {
        /// Button label
        label: String,
        /// Button payload
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Callback button
    #[serde(rename = "callback")]
    Callback {
        /// Button label
        label: String,
        /// Button payload
        payload: Value,
    },
    /// Link button
    #[serde(rename = "open_link")]
    OpenLink {
        /// Link URL
        link: String,
        /// Button label
        label: String,
        /// Button payload
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Location button
    #[serde(rename = "location")]
    Location {
        /// Button payload
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// VK Pay button
    #[serde(rename = "vkpay")]
    VkPay {
        /// Payment hash
        hash: String,
        /// Button payload
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    /// Open app button
    #[serde(rename = "open_app")]
    OpenApp {
        /// App ID
        app_id: i64,
        /// App owner ID
        owner_id: i64,
        /// Button label
        label: String,
        /// App hash
        hash: String,
        /// Button payload
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
}

/// Button color
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonColor {
    /// Primary color (blue)
    Primary,
    /// Secondary color (white)
    Secondary,
    /// Negative color (red)
    Negative,
    /// Positive color (green)
    Positive,
}

impl fmt::Display for ButtonColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ButtonColor::Primary => write!(f, "primary"),
            ButtonColor::Secondary => write!(f, "secondary"),
            ButtonColor::Negative => write!(f, "negative"),
            ButtonColor::Positive => write!(f, "positive"),
        }
    }
}

/// Keyboard button
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardButton {
    /// Button action
    pub action: ButtonAction,
    /// Button color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<ButtonColor>,
}

/// Keyboard structure
#[derive(Debug, Clone)]
pub struct Keyboard {
    buttons: Vec<Vec<KeyboardButton>>,
    one_time: bool,
    inline: bool,
}

impl Keyboard {
    /// Create new empty keyboard
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            one_time: false,
            inline: false,
        }
    }

    /// Create new inline keyboard
    pub fn new_inline() -> Self {
        Self {
            buttons: Vec::new(),
            one_time: false,
            inline: true,
        }
    }

    /// Create new one-time keyboard
    pub fn new_one_time() -> Self {
        Self {
            buttons: Vec::new(),
            one_time: true,
            inline: false,
        }
    }

    /// Add row of buttons
    pub fn add_row(mut self, row: Vec<KeyboardButton>) -> Self {
        self.buttons.push(row);
        self
    }

    /// Add text button to last row
    pub fn add_text_button(
        mut self,
        label: &str,
        payload: Option<Value>,
        color: Option<ButtonColor>,
    ) -> Self {
        if self.buttons.is_empty() {
            self.buttons.push(Vec::new());
        }

        let last_row = self.buttons.last_mut().unwrap();
        last_row.push(KeyboardButton {
            action: ButtonAction::Text {
                label: label.to_string(),
                payload,
            },
            color,
        });

        self
    }

    /// Add callback button to last row
    pub fn add_callback_button(
        mut self,
        label: &str,
        payload: Value,
        color: Option<ButtonColor>,
    ) -> Self {
        if self.buttons.is_empty() {
            self.buttons.push(Vec::new());
        }

        let last_row = self.buttons.last_mut().unwrap();
        last_row.push(KeyboardButton {
            action: ButtonAction::Callback {
                label: label.to_string(),
                payload,
            },
            color,
        });

        self
    }

    /// Add command button to last row
    pub fn add_command_button(
        mut self,
        label: &str,
        payload: Value,
        color: Option<ButtonColor>,
    ) -> Self {
        if self.buttons.is_empty() {
            self.buttons.push(Vec::new());
        }

        let last_row = self.buttons.last_mut().unwrap();
        last_row.push(KeyboardButton {
            action: ButtonAction::Callback {
                label: label.to_owned(),
                payload: json!({"command": payload}),
            },
            color,
        });

        self
    }

    /// Add link button to last row
    pub fn add_link_button(
        mut self,
        label: &str,
        link: &str,
        payload: Option<Value>,
        color: Option<ButtonColor>,
    ) -> Self {
        if self.buttons.is_empty() {
            self.buttons.push(Vec::new());
        }

        let last_row = self.buttons.last_mut().unwrap();
        last_row.push(KeyboardButton {
            action: ButtonAction::OpenLink {
                link: link.to_owned(),
                label: label.to_owned(),
                payload,
            },
            color,
        });

        self
    }

    /// Convert to JSON value
    pub fn to_json(&self) -> Value {
        let buttons_json: Vec<Value> = self
            .buttons
            .iter()
            .map(|row| {
                let row_json: Vec<Value> = row
                    .iter()
                    .map(|button| {
                        let mut button_json = json!({
                            "action": button.action,
                        });

                        if let Some(color) = &button.color {
                            button_json["color"] = json!(color.to_string());
                        }

                        button_json
                    })
                    .collect();

                json!(row_json)
            })
            .collect();

        json!({
            "one_time": self.one_time,
            "inline": self.inline,
            "buttons": buttons_json,
        })
    }

    /// Convert to JSON string
    pub fn to_json_string(&self) -> String {
        self.to_json().to_string()
    }

    /// Create default menu keyboard
    pub fn create_menu() -> Self {
        Keyboard::new_inline()
            .add_text_button(
                "📋 Help",
                Some(json!({"command": "help"})),
                Some(ButtonColor::Primary),
            )
            .add_text_button(
                "ℹ️ Info",
                Some(json!({"command": "info"})),
                Some(ButtonColor::Secondary),
            )
            .add_row(Vec::new()) // New row
            .add_text_button(
                "🎲 Random",
                Some(json!({"command": "random"})),
                Some(ButtonColor::Positive),
            )
            .add_text_button(
                "🕒 Time",
                Some(json!({"command": "time"})),
                Some(ButtonColor::Negative),
            )
    }

    /// Create admin menu keyboard
    pub fn create_admin_menu() -> Self {
        Keyboard::new_inline()
            .add_text_button(
                "📊 Stats",
                Some(json!({"command": "stats"})),
                Some(ButtonColor::Primary),
            )
            .add_text_button(
                "📢 Broadcast",
                Some(json!({"command": "broadcast"})),
                Some(ButtonColor::Secondary),
            )
            .add_row(Vec::new())
            .add_text_button(
                "🚫 Ban",
                Some(json!({"command": "ban"})),
                Some(ButtonColor::Negative),
            )
            .add_text_button(
                "✅ Unban",
                Some(json!({"command": "unban"})),
                Some(ButtonColor::Positive),
            )
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
