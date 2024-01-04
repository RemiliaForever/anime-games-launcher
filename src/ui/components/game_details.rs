use std::process::Command;

use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

use anime_game_core::filesystem::DriverExt;

use crate::games;
use crate::config;

use crate::components::wine::Wine;

use crate::ui::components::game_card::{
    CardInfo,
    CardComponent,
    CardComponentInput
};

#[derive(Debug)]
pub struct GameDetailsComponent {
    pub game_card: AsyncController<CardComponent>,

    pub info: CardInfo,
    pub installed: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentInput {
    SetInfo(CardInfo),
    SetInstalled(bool),
    EditCard(CardComponentInput),

    EmitDownloadGame,
    EmitVerifyGame,

    LaunchGame
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentOutput {
    HideDetails,
    ShowTasksFlap,

    DownloadGame(CardInfo),
    VerifyGame(CardInfo),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetailsComponent {
    type Init = CardInfo;
    type Input = GameDetailsComponentInput;
    type Output = GameDetailsComponentOutput;

    view! {
        #[root]
        gtk::Box {
            set_valign: gtk::Align::Center,
            set_halign: gtk::Align::Center,

            set_vexpand: true,

            model.game_card.widget(),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,

                set_margin_start: 64,

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    add_css_class: "title-1",

                    #[watch]
                    set_label: model.info.get_title()
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_margin_top: 8,

                    #[watch]
                    set_label: &format!("Developer: {}", model.info.get_developer())
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: model.installed,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
    
                        set_label: "Played: 4,837 hours"
                    },
    
                    gtk::Label {
                        set_halign: gtk::Align::Start,
    
                        set_label: "Last played: yesterday"
                    },

                    gtk::Box {
                        set_valign: gtk::Align::Center,
    
                        set_margin_top: 36,
                        set_spacing: 8,
    
                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
    
                            #[watch]
                            set_visible: model.installed,
    
                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",
                                set_label: "Play"
                            },

                            connect_clicked => GameDetailsComponentInput::LaunchGame
                        },
    
                        gtk::Button {
                            add_css_class: "pill",
    
                            #[watch]
                            set_visible: model.installed,
    
                            adw::ButtonContent {
                                set_icon_name: "drive-harddisk-ieee1394-symbolic",
                                set_label: "Verify"
                            },

                            connect_clicked => GameDetailsComponentInput::EmitVerifyGame
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: !model.installed,

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 36,
                        set_spacing: 8,

                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "suggested-action",

                            adw::ButtonContent {
                                set_icon_name: "folder-download-symbolic",
                                set_label: "Download"
                            },

                            connect_clicked => GameDetailsComponentInput::EmitDownloadGame
                        }
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            game_card: CardComponent::builder()
                .launch(init.clone())
                .detach(),

            info: init,
            installed: false
        };

        model.game_card.emit(CardComponentInput::SetClickable(false));
        model.game_card.emit(CardComponentInput::SetDisplayTitle(false));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsComponentInput::SetInfo(info) => {
                self.info = info.clone();

                self.game_card.emit(CardComponentInput::SetInfo(info));
            }

            GameDetailsComponentInput::SetInstalled(installed) => {
                self.installed = installed;

                self.game_card.emit(CardComponentInput::SetInstalled(installed));
            }

            GameDetailsComponentInput::EditCard(message) => self.game_card.emit(message),

            GameDetailsComponentInput::EmitDownloadGame => {
                sender.output(GameDetailsComponentOutput::DownloadGame(self.info.clone())).unwrap();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
                sender.output(GameDetailsComponentOutput::ShowTasksFlap).unwrap();
            }

            GameDetailsComponentInput::EmitVerifyGame => {
                sender.output(GameDetailsComponentOutput::VerifyGame(self.info.clone())).unwrap();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
                sender.output(GameDetailsComponentOutput::ShowTasksFlap).unwrap();
            }

            GameDetailsComponentInput::LaunchGame => {
                fn launch(info: &CardInfo) -> anyhow::Result<()> {
                    // Get game driver
                    let Some(game) = games::get(info.get_name())? else {
                        anyhow::bail!("Unable to find {} integration script", info.get_title());
                    };

                    // Get game settings
                    let settings = config::get().games.get_game_settings(info.get_name())?;

                    // Get installation folder driver
                    let Some(driver) = settings.paths.get(info.get_edition()) else {
                        anyhow::bail!("Unable to find {} installation path", info.get_title());
                    };

                    let driver = driver.to_dyn_trait();

                    // Deploy the game
                    let path = driver.deploy()?;

                    // Request game launch options
                    let options = game.get_launch_options(path.to_string_lossy())?;

                    // Prepare launch command
                    let command = [
                        format!("{:?}", Wine::from_config()?.get_executable()),
                        format!("{:?}", path.join(options.executable))
                    ];

                    // Run the game
                    Command::new("bash")
                        .arg("-c")
                        .arg(command.join(" "))
                        .envs(options.environment)
                        .current_dir(path)
                        .spawn()?
                        .wait()?;

                    // Dismantle installation fodler
                    driver.dismantle()?;

                    Ok(())
                }

                let info = self.info.clone();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();

                std::thread::spawn(move || {
                    if let Err(err) = launch(&info) {
                        sender.output(GameDetailsComponentOutput::ShowToast {
                            title: format!("Failed to launch {}", info.get_title()),
                            message: Some(err.to_string())
                        }).unwrap();
                    }
                });
            }
        }
    }
}
