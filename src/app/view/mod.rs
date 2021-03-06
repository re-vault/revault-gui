pub mod charging;
mod deposit;
mod emergency;
mod home;
mod layout;
pub mod manager;
mod network;
mod settings;
mod sidebar;
pub mod sign;
pub mod spend_transaction;
pub mod stakeholder;
pub mod vault;
mod vaults;

pub use deposit::DepositView;
pub use emergency::EmergencyView;
pub use home::{ManagerHomeView, StakeholderHomeView};
pub use network::{ManagerNetworkView, StakeholderNetworkView};
pub use settings::SettingsView;
pub use spend_transaction::{SpendTransactionListItemView, SpendTransactionView};
pub use stakeholder::{StakeholderCreateVaultsView, StakeholderDelegateFundsView};
pub use vault::VaultView;
pub use vaults::VaultsView;

use bitcoin::Network;

use super::menu::Menu;
use crate::{conversion::Converter, revault::Role};

/// Context stores display informations and features
/// used directly by views. It does not store anything
/// related to Revault logic.
pub struct Context {
    pub converter: Converter,
    pub network: Network,
    pub network_up: bool,
    pub menu: Menu,
    pub role: Role,
    pub role_edit: bool,
}

impl Context {
    pub fn new(
        converter: Converter,
        network: Network,
        role_edit: bool,
        role: Role,
        menu: Menu,
    ) -> Self {
        Self {
            converter,
            role,
            role_edit,
            menu,
            network,
            network_up: false,
        }
    }
}

impl std::default::Default for Context {
    fn default() -> Self {
        Context {
            converter: Converter::new(Network::Bitcoin),
            network: Network::Bitcoin,
            network_up: false,
            role: Role::Manager,
            menu: Menu::Home,
            role_edit: false,
        }
    }
}
