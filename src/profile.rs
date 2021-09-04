use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub struct Emote {
    pub name: String,
    pub url: String,
}

impl Emote {
    pub fn to_action(&self, prefix: &str, include_label: bool) -> Action {
        let mut state = State::default();

        if include_label {
            state.title = self.name.clone();
        }

        let mut name = self.name.clone();
        if !prefix.is_empty() && !name.is_empty() {
            if let Some(c) = name.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
        }

        Action {
            name: "Text".into(),
            state: 0,
            states: vec![state],
            settings: Settings::Text {
                is_sending_enter: false,
                pasted_text: format!(":_{}{}:", prefix, name),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProfileManifest {
    pub actions: HashMap<Position, Action>,
    pub device_model: DeviceModel,
    #[serde(rename = "DeviceUUID")]
    pub device_uuid: String, // e.g., `@(1)[4057/128/DL16K1A70561]`
    pub name: String,
    pub version: String, // `1.0`
}

pub enum DeviceModel {
    Standard,
    XL,
    Mini,
}

impl DeviceModel {
    pub fn id(&self) -> &'static str {
        match self {
            Self::Standard => "20GBA9901",
            Self::XL => "20GAT9901",
            Self::Mini => "unknown", // TODO: Find correct value
        }
    }

    pub fn size(&self) -> (u8, u8) {
        match self {
            Self::Standard => (5, 3),
            Self::XL => (4, 8),
            Self::Mini => (3, 2),
        }
    }
}
impl Serialize for DeviceModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.id())
    }
}

impl ProfileManifest {
    pub fn set_actions(&mut self, actions: Vec<Option<Action>>) {
        let (width, height) = self.device_model.size();

        for (index, action) in actions.into_iter().enumerate() {
            let index = index as u8;
            let pos = Position::new(index % width, index / height);

            if let Some(action) = action {
                self.actions.insert(pos, action);
            } else {
                self.actions.remove(&pos);
            }
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{},{}", self.x, self.y)
    }
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Action {
    pub state: u8,
    pub states: Vec<State>,
    pub name: String,
    #[serde(flatten)]
    pub settings: Settings,
}

#[derive(Serialize, Debug)]
#[serde(tag = "UUID", content = "Settings", rename_all = "PascalCase")]
pub enum Settings {
    #[serde(rename = "com.elgato.streamdeck.profile.backtoparent")]
    BackToParent {},
    #[serde(rename = "com.elgato.streamdeck.profile.openchild")]
    OpenChild {
        #[serde(rename = "ProfileUUID", serialize_with = "uuid_uppercase")]
        profile_uuid: Uuid,
    },
    #[serde(rename = "com.elgato.streamdeck.system.text", rename_all = "camelCase")]
    Text {
        is_sending_enter: bool,
        pasted_text: String,
    },
}

fn uuid_uppercase<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string().to_uppercase())
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct State {
    pub f_family: String,
    pub f_size: String,
    pub f_style: String,
    pub f_underline: String,
    pub image: String,
    pub title: String,
    pub title_alignment: String,
    pub title_color: String,
    pub title_show: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            f_family: "".into(),
            f_size: "12".into(),
            f_style: "".into(),
            f_underline: "off".into(),
            image: "state0.png".into(),
            title: "".into(),
            title_alignment: "bottom".into(),
            title_color: "#fbfcff".into(),
            title_show: "".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn serialize_profile() -> Result<()> {
        let mut actions = HashMap::new();

        actions.insert(
            Position::new(0, 0),
            Action {
                name: "Open Folder".into(),
                state: 0,
                states: vec![State::default()],
                settings: Settings::BackToParent {},
            },
        );

        actions.insert(
            Position::new(1, 0),
            Action {
                name: "Text".into(),
                state: 0,
                states: vec![State::default()],
                settings: Settings::Text {
                    is_sending_enter: false,
                    pasted_text: ":_pomuSmall9cm:".into(),
                },
            },
        );

        let profile_uuid = Uuid::parse_str("AC20BCF3-0A7C-4243-BB74-5C0DC5681BA5")?;

        actions.insert(
            Position::new(4, 2),
            Action {
                name: "Create Folder".into(),
                state: 0,
                states: vec![State::default()],
                settings: Settings::OpenChild { profile_uuid },
            },
        );

        let profile = ProfileManifest {
            actions,
            device_model: DeviceModel::Standard,
            device_uuid: "@(1)[4057/128/DL16K1A71331]".into(),
            name: "Emotes".into(),
            version: "1.0".into(),
        };

        let json = serde_json::to_value(&profile)?;

        let expected = serde_json::json!({
          "Actions": {
            "0,0": {
              "State": 0,
              "States": [
                {
                  "FFamily": "",
                  "FSize": "12",
                  "FStyle": "",
                  "FUnderline": "off",
                  "Image": "state0.png",
                  "Title": "",
                  "TitleAlignment": "bottom",
                  "TitleColor": "#fbfcff",
                  "TitleShow": ""
                }
              ],
              "Name": "Open Folder",
              "UUID": "com.elgato.streamdeck.profile.backtoparent",
              "Settings": {}
            },
            "1,0": {
              "State": 0,
              "States": [
                {
                  "FFamily": "",
                  "FSize": "12",
                  "FStyle": "",
                  "FUnderline": "off",
                  "Image": "state0.png",
                  "Title": "",
                  "TitleAlignment": "bottom",
                  "TitleColor": "#fbfcff",
                  "TitleShow": ""
                }
              ],
              "Name": "Text",
              "UUID": "com.elgato.streamdeck.system.text",
              "Settings": {
                "isSendingEnter": false,
                "pastedText": ":_pomuSmall9cm:"
              }
            },
            "4,2": {
              "State": 0,
              "States": [
                {
                  "FFamily": "",
                  "FSize": "12",
                  "FStyle": "",
                  "FUnderline": "off",
                  "Image": "state0.png",
                  "Title": "",
                  "TitleAlignment": "bottom",
                  "TitleColor": "#fbfcff",
                  "TitleShow": ""
                }
              ],
              "Name": "Create Folder",
              "UUID": "com.elgato.streamdeck.profile.openchild",
              "Settings": {
                "ProfileUUID": "AC20BCF3-0A7C-4243-BB74-5C0DC5681BA5"
              }
            }
          },
          "DeviceModel": "20GBA9901",
          "DeviceUUID": "@(1)[4057/128/DL16K1A71331]",
          "Name": "Emotes",
          "Version": "1.0"
        });

        assert_eq!(json, expected);

        Ok(())
    }

    #[test]
    fn emote_to_action_with_prefix() -> Result<()> {
        let emote = Emote {
            url: "http://example.com/image.png".into(),
            name: "small9cm".into(),
        };

        let action = emote.to_action("pomu", true);

        assert_eq!(action.states[0].title, "small9cm");

        match action.settings {
            Settings::Text { pasted_text, .. } if pasted_text == ":_pomuSmall9cm:" => {}
            _ => anyhow::bail!(
                "Failed to find expected text in settings: {:?}",
                action.settings
            ),
        }

        Ok(())
    }

    #[test]
    fn emote_to_action_no_prefix() -> Result<()> {
        let emote = Emote {
            url: "http://example.com/image.png".into(),
            name: "hic1".into(),
        };

        let action = emote.to_action("", false);

        assert_eq!(action.states[0].title, "");

        match action.settings {
            Settings::Text { pasted_text, .. } if pasted_text == ":_hic1:" => {}
            _ => anyhow::bail!(
                "Failed to find expected text in settings: {:?}",
                action.settings
            ),
        }

        Ok(())
    }
}
