use crate::{
    installer::{message, view},
    ui::component::form,
};

use iced::{button::State as Button, text_input, Element};

#[derive(Clone)]
pub struct ParticipantXpub {
    pub xpub: form::Value<String>,

    xpub_input: text_input::State,
    delete_button: Button,
}

impl ParticipantXpub {
    pub fn new() -> Self {
        Self {
            xpub: form::Value::default(),
            xpub_input: text_input::State::new(),
            delete_button: Button::new(),
        }
    }

    pub fn update(&mut self, msg: message::ParticipantXpub) {
        if let message::ParticipantXpub::XpubEdited(xpub) = msg {
            self.xpub.value = xpub;
            self.xpub.valid = true;
        }
    }

    pub fn view(&mut self) -> Element<message::ParticipantXpub> {
        view::participant_xpub(&self.xpub, &mut self.xpub_input, &mut self.delete_button)
    }
}

pub struct CosignerKey {
    pub key: form::Value<String>,

    key_input: text_input::State,
    delete_button: Button,
}

impl CosignerKey {
    pub fn new() -> Self {
        Self {
            key: form::Value::default(),
            key_input: text_input::State::new(),
            delete_button: Button::new(),
        }
    }

    pub fn update(&mut self, msg: message::CosignerKey) {
        if let message::CosignerKey::KeyEdited(key) = msg {
            self.key.value = key;
            self.key.valid = true;
        }
    }

    pub fn view(&mut self) -> Element<message::CosignerKey> {
        view::cosigner_key(&self.key, &mut self.key_input, &mut self.delete_button)
    }
}
