use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Profile {
    pub actions: HashMap<Position, Action>,
    pub device_model: String,
    #[serde(rename = "DeviceUUID")]
    pub device_uuid: String,
    pub name: String,
    pub version: String,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    fn new(x: u8, y: u8) -> Self {
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

#[derive(Serialize)]
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

        let profile = Profile {
            actions,
            device_model: "20GBA9901".into(),
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
}
