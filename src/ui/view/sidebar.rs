use iced::{pick_list, Column, Container, Length, Row};

use crate::ui::{
    component::{button, separation, text, TransparentPickListStyle},
    icon::{dot_icon, history_icon, home_icon, network_icon, send_icon, settings_icon},
    message::{Context, Menu, Message, Role},
    view::layout,
};

#[derive(Debug, Clone)]
pub struct Sidebar {
    pick_role: pick_list::State<Role>,
    home_menu_button: iced::button::State,
    history_menu_button: iced::button::State,
    network_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            home_menu_button: iced::button::State::new(),
            history_menu_button: iced::button::State::new(),
            network_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
            pick_role: pick_list::State::default(),
        }
    }

    pub fn view(&mut self, context: &Context) -> Container<Message> {
        let role = if context.role_edit {
            Container::new(
                pick_list::PickList::new(
                    &mut self.pick_role,
                    &Role::ALL[..],
                    Some(context.role),
                    Message::ChangeRole,
                )
                .padding(10)
                .width(Length::Units(200))
                .style(TransparentPickListStyle),
            )
        } else {
            Container::new(text::simple(&format!("{}", context.role)))
        };
        let home_button = if context.menu == Menu::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
                Message::Menu(Menu::Home),
            )
        };
        let history_button = if context.menu == Menu::History {
            button::primary(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
                Message::Menu(Menu::History),
            )
        } else {
            button::transparent(
                &mut self.history_menu_button,
                button::button_content(Some(history_icon()), "History"),
                Message::Menu(Menu::History),
            )
        };
        let network_button = if context.menu == Menu::Network {
            button::primary(
                &mut self.network_menu_button,
                button::button_content(Some(network_icon()), "Network"),
                Message::Menu(Menu::Network),
            )
        } else {
            let mut row = Row::new()
                .push(network_icon())
                .push(text::simple("Network"))
                .spacing(10)
                .align_items(iced::Align::Center);

            if context.network_up {
                row = row.push(text::success(dot_icon().size(7)))
            } else {
                row = row.push(text::danger(dot_icon().size(7)))
            }

            button::transparent(
                &mut self.network_menu_button,
                Container::new(row).padding(5),
                Message::Menu(Menu::Network),
            )
        };
        let actions = if context.role == Role::Manager {
            Container::new(
                button::transparent(
                    &mut self.spend_menu_button,
                    button::button_content(Some(send_icon()), "Send"),
                    Message::Menu(Menu::Send),
                )
                .width(iced::Length::Units(200)),
            )
        } else {
            Container::new(Column::new())
        };
        layout::sidebar(
            layout::sidebar_menu(vec![
                role.width(Length::Units(200)),
                separation().width(iced::Length::Units(200)),
                Container::new(home_button.width(Length::Units(200))),
                Container::new(history_button.width(Length::Units(200))),
                Container::new(network_button.width(Length::Units(200))),
                separation().width(Length::Units(200)),
                actions,
            ]),
            Container::new(
                button::transparent(
                    &mut self.settings_menu_button,
                    button::button_content(Some(settings_icon()), "Settings"),
                    Message::Install,
                )
                .width(iced::Length::Units(200)),
            ),
        )
    }
}
