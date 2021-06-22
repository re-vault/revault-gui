mod common;
pub mod manager;
pub mod stakeholder;

use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, Element};
use miniscript::DescriptorPublicKey;
use revault_tx::scripts::CpfpDescriptor;

use crate::{
    installer::{
        message::{self, Message},
        step::common::ParticipantXpub,
        view,
    },
    revaultd::config,
};

pub trait Step {
    fn update(&mut self, message: Message);
    fn view(&mut self) -> Element<Message>;
    fn load_context(&mut self, _ctx: &Context) {}
    fn apply(&mut self, _ctx: &mut Context, _config: &mut config::Config) -> bool {
        true
    }
}

pub struct Context {
    pub number_cosigners: usize,
    pub stakeholders_xpubs: Vec<String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            number_cosigners: 0,
            stakeholders_xpubs: Vec::new(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Welcome {
    install_button: Button,
}

impl Welcome {
    pub fn new() -> Self {
        Self {
            install_button: Button::new(),
        }
    }
}

impl Step for Welcome {
    fn update(&mut self, _message: Message) {}
    fn view(&mut self) -> Element<Message> {
        view::welcome(&mut self.install_button)
    }
}

impl Default for Welcome {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Welcome> for Box<dyn Step> {
    fn from(s: Welcome) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineRole {
    stakeholder_button: Button,
    manager_button: Button,
    stakeholder_manager_button: Button,
    scroll: scrollable::State,
}

impl DefineRole {
    pub fn new() -> Self {
        Self {
            stakeholder_button: Button::new(),
            manager_button: Button::new(),
            stakeholder_manager_button: Button::new(),
            scroll: scrollable::State::new(),
        }
    }
}

impl Step for DefineRole {
    fn update(&mut self, _message: Message) {}
    fn view(&mut self) -> Element<Message> {
        view::define_role(
            &mut self.stakeholder_button,
            &mut self.manager_button,
            &mut self.stakeholder_manager_button,
            &mut self.scroll,
        )
    }
}

impl Default for DefineRole {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineRole> for Box<dyn Step> {
    fn from(s: DefineRole) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineCpfpDescriptor {
    manager_xpubs: Vec<ParticipantXpub>,
    add_xpub_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
    warning: Option<String>,
}

impl DefineCpfpDescriptor {
    pub fn new() -> Self {
        Self {
            add_xpub_button: Button::new(),
            manager_xpubs: Vec::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
            warning: None,
        }
    }
}

impl Step for DefineCpfpDescriptor {
    fn update(&mut self, message: Message) {
        if let Message::DefineCpfpDescriptor(msg) = message {
            match msg {
                message::DefineCpfpDescriptor::ManagerXpub(i, message::ParticipantXpub::Delete) => {
                    self.manager_xpubs.remove(i);
                }
                message::DefineCpfpDescriptor::ManagerXpub(i, msg) => {
                    if let Some(xpub) = self.manager_xpubs.get_mut(i) {
                        xpub.update(msg);
                    }
                }
                message::DefineCpfpDescriptor::AddXpub => {
                    self.manager_xpubs.push(ParticipantXpub::new());
                }
            };
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.manager_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }

        if self.manager_xpubs.iter().any(|xpub| xpub.warning) {
            return false;
        }

        let mut xpubs: Vec<String> = self
            .manager_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub))
            .collect();

        xpubs.sort();

        let keys = xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        match CpfpDescriptor::new(keys) {
            Ok(descriptor) => config.scripts_config.cpfp_descriptor = descriptor.to_string(),
            Err(e) => self.warning = Some(e.to_string()),
        }

        self.warning.is_none()
    }

    fn view(&mut self) -> Element<Message> {
        return view::define_cpfp_descriptor(
            &mut self.add_xpub_button,
            self.manager_xpubs
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineCpfpDescriptor(message::DefineCpfpDescriptor::ManagerXpub(
                            i, msg,
                        ))
                    })
                })
                .collect(),
            &mut self.scroll,
            &mut self.previous_button,
            &mut self.save_button,
            self.warning.as_ref(),
        );
    }
}

impl Default for DefineCpfpDescriptor {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineCpfpDescriptor> for Box<dyn Step> {
    fn from(s: DefineCpfpDescriptor) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineCoordinator {
    host: String,
    noise_key: String,
    warning: bool,

    view: view::DefineCoordinator,
}

impl DefineCoordinator {
    pub fn new() -> Self {
        Self {
            host: "".to_string(),
            noise_key: "".to_string(),
            warning: false,
            view: view::DefineCoordinator::new(),
        }
    }
}

impl Step for DefineCoordinator {
    fn update(&mut self, message: Message) {
        if let Message::DefineCoordinator(msg) = message {
            match msg {
                message::DefineCoordinator::HostEdited(host) => self.host = host,
                message::DefineCoordinator::NoiseKeyEdited(key) => self.noise_key = key,
            };
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        config.coordinator_host = self.host.clone();
        config.coordinator_noise_key = self.noise_key.clone();
        true
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(&self.host, &self.noise_key, self.warning)
    }
}

impl Default for DefineCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineCoordinator> for Box<dyn Step> {
    fn from(s: DefineCoordinator) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineBitcoind {
    network: bitcoin::Network,
    cookie_path: String,
    address: String,

    warning_cookie: bool,
    warning_address: bool,

    view: view::DefineBitcoind,
}

impl DefineBitcoind {
    pub fn new() -> Self {
        Self {
            network: bitcoin::Network::Bitcoin,
            cookie_path: "".to_string(),
            address: "".to_string(),
            warning_cookie: false,
            warning_address: false,
            view: view::DefineBitcoind::new(),
        }
    }
}

impl Step for DefineBitcoind {
    fn update(&mut self, message: Message) {
        if let Message::DefineBitcoind(msg) = message {
            match msg {
                message::DefineBitcoind::AddressEdited(address) => {
                    self.address = address;
                    self.warning_address = false;
                }
                message::DefineBitcoind::CookiePathEdited(path) => {
                    self.cookie_path = path;
                    self.warning_cookie = false;
                }
                message::DefineBitcoind::NetworkEdited(network) => {
                    self.network = network;
                }
            };
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        match (
            PathBuf::from_str(&self.cookie_path),
            std::net::SocketAddr::from_str(&self.address),
        ) {
            (Err(_), Ok(_)) => {
                self.warning_cookie = true;
                false
            }
            (Ok(_), Err(_)) => {
                self.warning_address = true;
                false
            }
            (Err(_), Err(_)) => {
                self.warning_address = true;
                self.warning_cookie = true;
                false
            }
            (Ok(path), Ok(addr)) => {
                config.bitcoind_config = config::BitcoindConfig {
                    network: self.network,
                    cookie_path: path,
                    poll_interval_secs: None,
                    addr,
                };
                true
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            &self.network,
            &self.address,
            &self.cookie_path,
            self.warning_address,
            self.warning_cookie,
        )
    }
}

impl Default for DefineBitcoind {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineBitcoind> for Box<dyn Step> {
    fn from(s: DefineBitcoind) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct Final {
    generating: bool,
    warning: Option<String>,
    config_path: Option<PathBuf>,
    view: view::Final,
}

impl Final {
    pub fn new() -> Self {
        Self {
            generating: false,
            warning: None,
            config_path: None,
            view: view::Final::new(),
        }
    }
}

impl Step for Final {
    fn update(&mut self, message: Message) {
        match message {
            Message::Installed(res) => {
                self.generating = false;
                match res {
                    Err(e) => {
                        self.config_path = None;
                        self.warning = Some(e.to_string());
                    }
                    Ok(path) => self.config_path = Some(path),
                }
            }
            Message::Install => {
                self.generating = true;
                self.config_path = None;
                self.warning = None;
            }
            _ => {}
        };
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            self.generating,
            self.config_path.as_ref(),
            self.warning.as_ref(),
        )
    }
}

impl Default for Final {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Final> for Box<dyn Step> {
    fn from(s: Final) -> Box<dyn Step> {
        Box::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::{DefineCpfpDescriptor as DefineCpfpDescriptorStep, *};
    use crate::installer::message::{DefineCpfpDescriptor, ParticipantXpub, *};
    use crate::revaultd::config::Config;

    const STAKEHOLDERS_XPUBS: [&str; 4] = [
        "xpub6DEzq5DNPx2rPiZJ7wvFhxRKUKDoV1GwjFmFdaxFfbsw9HsHyxc9usoRUMxqJaMrwoXh4apahsGEnjAS4cVCBDgqsx5Groww22AdHbgxVDg", 
        "xpub6F7Ltmsut73cbUNAzh44DkxncMeQfPtRzx7aoXjFbUdd7yofR2intU4b6QcsXot1jgmVjHB3iMybCLhtqvhAx3L4VPbGUz5fwuyNeTkypUP",
        "xpub6CutNDrGhiD8GbjgKQWoTfzdRmoHJT8AcBxaV4NvWmo4dE5KKwpg2ukvgiCRwgZuJRXxKRsgRrrZiDZFJw1rLyAvY7X52WNEuaJXcVKLVFG", 
        "xpub6EN35Df8V826n4HuW4QZEhFyyMq4jmou3AFnVqRpoFw8YS68ojkVNzVGWhnkCyGwZjVVUEoeBWhTfJ38C3Fvsc3ibvYFi5BvmQwAMZkqEqH"
    ];

    const MANAGERS_XPUBS: [&str; 2] = [
        "xpub6CZFHPW1GiB8YgV7zGpeQDB6mMHZYPQyUaHrM1nMvKMgLxwok4xCtnzjuxQ3p1LHJUkz5i1Y7bRy5fmGrdg8UBVb39XdXNtWWd2wTsNd7T9",
        "xpub6Doj75MBvKp7bgHxF1KeDGxm36rd4wonZWv8sfzTeNoNVX2QZaQdrEcs7NDXvs4Cbsy9TPMx5VDcMK6JjSKepBbYDPiJ9bLBR4bqfdHmxZx",
    ];

    const COSIGNERS_KEYS: [&str; 4] = [
        "030f64b922aee2fd597f104bc6cb3b670f1ca2c6c49b1071a1a6c010575d94fe5a",
        "02abe475b199ec3d62fa576faee16a334fdb86ffb26dce75becebaaedf328ac3fe",
        "0314f3dc33595b0d016bb522f6fe3a67680723d842c1b9b8ae6b59fdd8ab5cccb4",
        "025eba3305bd3c829e4e1551aac7358e4178832c739e4fc4729effe428de0398ab",
    ];

    fn load_stakeholders_xpubs(step: &mut dyn Step, xpubs: Vec<String>) {
        let mut i = 0;
        for xpub in xpubs {
            step.update(Message::DefineStakeholderXpubs(
                DefineStakeholderXpubs::AddXpub,
            ));
            step.update(Message::DefineStakeholderXpubs(
                DefineStakeholderXpubs::StakeholderXpub(i, ParticipantXpub::XpubEdited(xpub)),
            ));
            i += 1;
        }
    }

    fn load_managers_xpubs(step: &mut dyn Step, xpubs: Vec<String>) {
        let mut i = 0;
        for xpub in xpubs {
            step.update(Message::DefineManagerXpubs(DefineManagerXpubs::AddXpub));
            step.update(Message::DefineManagerXpubs(
                DefineManagerXpubs::ManagerXpub(i, ParticipantXpub::XpubEdited(xpub)),
            ));
            i += 1;
        }
    }

    fn load_cosigners_keys(step: &mut dyn Step, keys: Vec<String>) {
        let mut i = 0;
        for key in keys {
            step.update(Message::DefineManagerXpubs(DefineManagerXpubs::AddCosigner));
            step.update(Message::DefineManagerXpubs(
                DefineManagerXpubs::CosignerKey(i, CosignerKey::KeyEdited(key)),
            ));
            i += 1;
        }
    }

    #[test]
    fn define_deposit_descriptor() {
        let mut ctx = Context::new();
        let mut manager_step = manager::DefineStakeholderXpubs::new();
        load_stakeholders_xpubs(
            &mut manager_step,
            vec![
                STAKEHOLDERS_XPUBS[2].to_string(),
                STAKEHOLDERS_XPUBS[1].to_string(),
                STAKEHOLDERS_XPUBS[0].to_string(),
                STAKEHOLDERS_XPUBS[3].to_string(),
            ],
        );

        let mut manager_config = Config::new();
        manager_step.apply(&mut ctx, &mut manager_config);

        let mut stakeholder_step = stakeholder::DefineStakeholderXpubs::new();
        load_stakeholders_xpubs(
            &mut stakeholder_step,
            vec![
                STAKEHOLDERS_XPUBS[3].to_string(),
                STAKEHOLDERS_XPUBS[0].to_string(),
                STAKEHOLDERS_XPUBS[1].to_string(),
            ],
        );
        stakeholder_step.update(Message::DefineStakeholderXpubs(
            DefineStakeholderXpubs::OurXpubEdited(STAKEHOLDERS_XPUBS[2].to_string()),
        ));

        let mut stakeholder_config = Config::new();
        stakeholder_step.apply(&mut ctx, &mut stakeholder_config);

        assert_eq!(
            manager_config.scripts_config.deposit_descriptor,
            stakeholder_config.scripts_config.deposit_descriptor,
        );
    }

    #[test]
    fn define_unvault_descriptor() {
        let mut ctx = Context::new();
        let mut manager_step = manager::DefineManagerXpubs::new();
        manager_step.load_context(&Context {
            number_cosigners: 4,
            stakeholders_xpubs: vec![
                STAKEHOLDERS_XPUBS[2].to_string(),
                STAKEHOLDERS_XPUBS[1].to_string(),
                STAKEHOLDERS_XPUBS[0].to_string(),
                STAKEHOLDERS_XPUBS[3].to_string(),
            ],
        });

        load_managers_xpubs(&mut manager_step, vec![MANAGERS_XPUBS[0].to_string()]);
        load_cosigners_keys(
            &mut manager_step,
            vec![
                COSIGNERS_KEYS[2].to_string(),
                COSIGNERS_KEYS[1].to_string(),
                COSIGNERS_KEYS[0].to_string(),
                COSIGNERS_KEYS[3].to_string(),
            ],
        );

        manager_step.update(Message::DefineManagerXpubs(
            DefineManagerXpubs::OurXpubEdited(MANAGERS_XPUBS[1].to_string()),
        ));

        manager_step.update(Message::DefineManagerXpubs(
            DefineManagerXpubs::ManagersTreshold(Action::Increment),
        ));
        manager_step.update(Message::DefineManagerXpubs(
            DefineManagerXpubs::SpendingDelay(Action::Increment),
        ));

        let mut manager_config = Config::new();
        manager_step.apply(&mut ctx, &mut manager_config);

        let mut stakeholder_step = stakeholder::DefineManagerXpubs::new();
        stakeholder_step.load_context(&Context {
            number_cosigners: 4,
            stakeholders_xpubs: vec![
                STAKEHOLDERS_XPUBS[3].to_string(),
                STAKEHOLDERS_XPUBS[2].to_string(),
                STAKEHOLDERS_XPUBS[0].to_string(),
                STAKEHOLDERS_XPUBS[1].to_string(),
            ],
        });

        load_managers_xpubs(
            &mut stakeholder_step,
            vec![MANAGERS_XPUBS[1].to_string(), MANAGERS_XPUBS[0].to_string()],
        );
        load_cosigners_keys(
            &mut stakeholder_step,
            vec![
                COSIGNERS_KEYS[3].to_string(),
                COSIGNERS_KEYS[2].to_string(),
                COSIGNERS_KEYS[0].to_string(),
                COSIGNERS_KEYS[1].to_string(),
            ],
        );
        stakeholder_step.update(Message::DefineManagerXpubs(
            DefineManagerXpubs::ManagersTreshold(Action::Increment),
        ));
        stakeholder_step.update(Message::DefineManagerXpubs(
            DefineManagerXpubs::SpendingDelay(Action::Increment),
        ));

        let mut stakeholder_config = Config::new();
        stakeholder_step.apply(&mut ctx, &mut stakeholder_config);

        assert_eq!(
            manager_config.scripts_config.unvault_descriptor,
            stakeholder_config.scripts_config.unvault_descriptor,
        );
    }

    #[test]
    fn define_cpfp_descriptor() {
        let mut ctx = Context::new();
        let mut cpfp_1_step = DefineCpfpDescriptorStep::new();
        cpfp_1_step.update(Message::DefineCpfpDescriptor(DefineCpfpDescriptor::AddXpub));
        cpfp_1_step.update(Message::DefineCpfpDescriptor(
            DefineCpfpDescriptor::ManagerXpub(
                0,
                ParticipantXpub::XpubEdited(MANAGERS_XPUBS[0].to_string()),
            ),
        ));
        cpfp_1_step.update(Message::DefineCpfpDescriptor(DefineCpfpDescriptor::AddXpub));
        cpfp_1_step.update(Message::DefineCpfpDescriptor(
            DefineCpfpDescriptor::ManagerXpub(
                1,
                ParticipantXpub::XpubEdited(MANAGERS_XPUBS[1].to_string()),
            ),
        ));

        let mut cpfp_1_config = Config::new();
        cpfp_1_step.apply(&mut ctx, &mut cpfp_1_config);

        let mut cpfp_2_step = DefineCpfpDescriptorStep::new();
        cpfp_2_step.update(Message::DefineCpfpDescriptor(DefineCpfpDescriptor::AddXpub));
        cpfp_2_step.update(Message::DefineCpfpDescriptor(
            DefineCpfpDescriptor::ManagerXpub(
                0,
                ParticipantXpub::XpubEdited(MANAGERS_XPUBS[1].to_string()),
            ),
        ));
        cpfp_2_step.update(Message::DefineCpfpDescriptor(DefineCpfpDescriptor::AddXpub));
        cpfp_2_step.update(Message::DefineCpfpDescriptor(
            DefineCpfpDescriptor::ManagerXpub(
                1,
                ParticipantXpub::XpubEdited(MANAGERS_XPUBS[0].to_string()),
            ),
        ));

        let mut cpfp_2_config = Config::new();
        cpfp_2_step.apply(&mut ctx, &mut cpfp_2_config);

        assert_eq!(
            cpfp_1_config.scripts_config.cpfp_descriptor,
            cpfp_2_config.scripts_config.cpfp_descriptor,
        );
    }
}
