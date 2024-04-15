use iced::{
    executor,
    widget::{
        container,
        text_editor::{Action, Content},
        Button, Column, PickList, Row, Space, Text, TextEditor, TextInput,
    },
    Application, Element, Length, Theme,
};
use iced_runtime::{core::text::editor::Edit, Command};
use std::{
    default::Default,
    fs,
    io::{Read, Write},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum Message {
    SelectConfig(String),
    ConfigName(String),
    ConfigData(Action),
    RefreshConfig,
    SaveConfig,
    Stop,
    Start,
    Console(Option<String>),
    ConsoleAction(Action),
}

pub struct WireGui {
    sudo: bool,
    config_list: Vec<String>,
    selected_config: Option<String>,
    config_name: String,
    config_data: Content,
    terminal: Content,
}

impl WireGui {
    fn list_configs() -> Vec<String> {
        let configs = match fs::read_dir("/etc/wireguard") {
            Ok(dir) => {
                dir.into_iter()
                    .filter_map(|entry| {
                        if let Ok(e) = entry {
                            let path = e.path();
                            if path.is_file()
                                && path.extension().and_then(|s| s.to_str()) == Some("conf")
                            {
                                Some(path.display().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            Err(_) => {
                vec![]
            }
        };

        configs
            .clone()
            .into_iter()
            .filter_map(|path| {
                path.split('/')
                    .last()
                    .and_then(|val| val.split('.').next().map(|v| v.to_string()))
            })
            .collect()
    }

    fn read_config(path: String) -> String {
        let path = format!("/etc/wireguard/{}.conf", path);
        if let Ok(mut file) = fs::File::open(path) {
            let mut out = "".to_string();
            if file.read_to_string(&mut out).is_ok() {
                return out;
            }
        }
        "".to_string()
    }

    fn write_config(path: String, data: String) {
        if data == *"\n" {
            return;
        }
        let path = format!("/etc/wireguard/{}.conf", path);
        if let Ok(mut file) = fs::File::create(path) {
            #[allow(clippy::collapsible_if)]
            if file.write_all(data.as_bytes()).is_ok() {
                if file.flush().is_err() {
                    log::debug!("Fail to flush the File!")
                }
            } else {
                log::debug!("Fail to write the File!")
            }
        }
    }

    async fn stop_wireguard(interface: String) -> Option<String> {
        let cmd = std::process::Command::new("sh")
            .arg("wg-quick")
            .arg("down")
            .arg(interface.clone())
            .output();

        if let Ok(ret) = cmd {
            if ret.status.success() {
                let out = format!("Stopped wireguard on interface {}!", interface);
                Some(out)
            } else {
                let out = format!("Fail to stop wireguard on interface {}!", interface);
                Some(out)
            }
        } else {
            Some("Fail to execute stop command!".to_string())
        }
    }

    async fn start_wireguard(interface: String) -> Option<String> {
        let cmd = std::process::Command::new("wg-quick")
            .arg("up")
            .arg(interface.clone())
            .output();

        if let Ok(ret) = cmd {
            if ret.status.success() {
                let out = format!("Started wireguard on interface {}!", interface);
                Some(out)
            } else {
                let out = format!("Fail to start wireguard on interface {}!", interface);
                Some(out)
            }
        } else {
            Some("Fail to execute start command!".to_string())
        }
    }

    fn sudo_display(&self) -> Element<'_, Message, Theme> {
        let configs = &self.config_list[..];

        let row = Row::new()
            .push(
                PickList::new(configs, self.selected_config.clone(), Message::SelectConfig)
                    .width(120),
            )
            .push(Space::with_width(5))
            .push(Button::new("").on_press(Message::RefreshConfig).width(32))
            .push(Space::with_width(5))
            .push(
                TextInput::new("new config name...", &self.config_name)
                    .on_input(Message::ConfigName)
                    .width(Length::Fill),
            )
            .push(Space::with_width(5))
            .push(Button::new("Save").on_press(Message::SaveConfig))
            .push(Space::with_width(5))
            .push(Button::new("Start").on_press(Message::Start))
            .push(Space::with_width(5))
            .push(Button::new("Stop").on_press(Message::Stop))
            .push(Space::with_width(5));

        let container = container(
            Column::new()
                .push(row)
                .push(Space::with_height(10))
                .push(
                    TextEditor::new(&self.config_data)
                        .on_action(Message::ConfigData)
                        .height(Length::Fill)
                        .padding(5),
                )
                .push(Space::with_height(10))
                .push(
                    TextEditor::new(&self.terminal)
                        .on_action(Message::ConsoleAction)
                        .height(200)
                        .padding(5),
                ),
        )
        .padding(5);

        container.into()
    }

    fn not_sudo_display(&self) -> Element<'_, Message, Theme> {
        Column::new()
            .push(Space::with_height(Length::Fill))
            .push(
                Row::new()
                    .push(Space::with_width(Length::Fill))
                    .push(Text::new("Run WireGUI as ROOT or SUDO!"))
                    .push(Space::with_width(Length::Fill)),
            )
            .push(Space::with_height(Length::Fill))
            .into()
    }

    fn update_configs(&mut self) {
        self.config_list = Self::list_configs();
    }

    fn select_config(&mut self, conf: String) {
        self.config_name = conf.clone();

        if self.config_list.contains(&conf) {
            self.selected_config = Some(conf.clone());
            // if the file exist
            let content = Self::read_config(conf);
            self.config_data = Content::new();
            self.config_data
                .perform(Action::Edit(Edit::Paste(Arc::new(content))));
        } else {
            self.selected_config = None;
            self.config_data = Content::new();
        };
    }

    fn save_config(&mut self) {
        let data = self.config_data.text();
        Self::write_config(self.config_name.clone(), data);
        self.update_configs();
    }
}

impl Application for WireGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_args: Self::Flags) -> (Self, Command<Self::Message>) {
        let w = WireGui {
            sudo: elevated_command::Command::is_elevated(),
            config_list: Self::list_configs(),
            selected_config: None,
            config_name: "".to_string(),
            config_data: Content::default(),
            terminal: Content::default(),
        };
        (w, Command::none())
    }

    fn title(&self) -> String {
        "WireGUI".to_string()
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        match event {
            Message::SelectConfig(config) => self.select_config(config),
            Message::ConfigName(name) => {
                self.update_configs();
                self.select_config(name);
            }
            Message::ConfigData(action) => self.config_data.perform(action),
            Message::RefreshConfig => self.update_configs(),
            Message::SaveConfig => self.save_config(),
            Message::Stop => {
                self.update_configs();
                if self.config_list.contains(&self.config_name) {
                    return Command::perform(
                        Self::stop_wireguard(self.config_name.clone()),
                        Message::Console,
                    );
                }
            }
            Message::Start => {
                self.update_configs();
                if self.config_list.contains(&self.config_name) {
                    return Command::perform(
                        Self::start_wireguard(self.config_name.clone()),
                        Message::Console,
                    );
                }
            }
            Message::Console(data) => {
                if let Some(data) = data {
                    let data = format!("{}\n", data);
                    self.terminal
                        .perform(Action::Edit(Edit::Paste(Arc::new(data))));
                }
            }
            Message::ConsoleAction(_) => {}
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message, Theme> {
        match self.sudo {
            true => self.sudo_display(),
            false => self.not_sudo_display(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
