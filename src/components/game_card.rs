use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::windows::main::MainAppMsg;
use crate::games::GameVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameCardComponent {
    pub width: i32,
    pub height: i32,
    pub variant: GameVariant,
    pub installed: bool,
    pub clickable: bool,
    pub display_title: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCardComponentInput {
    SetVariant(GameVariant),
    SetWidth(i32),
    SetHeight(i32),
    SetInstalled(bool),
    SetClickable(bool),
    SetDisplayTitle(bool),

    EmitCardClick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCardComponentOutput {
    CardClicked {
        variant: GameVariant,
        installed: bool
    }
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for GameCardComponent {
    type Init = GameVariant;
    type Input = GameCardComponentInput;
    type Output = GameCardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.width,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    gtk::Picture {
                        set_valign: gtk::Align::Start,
                        set_halign: gtk::Align::Start,

                        #[watch]
                        set_width_request: model.width,

                        #[watch]
                        set_height_request: model.height,

                        #[watch]
                        set_opacity: if model.installed {
                            1.0
                        } else {
                            0.4
                        },

                        add_css_class: "card",
                        add_css_class: "game-card",

                        // #[watch]
                        // set_css_classes: if model.installed {
                        //     &["card", "game-card"]
                        // } else {
                        //     &["card", "game-card", "game-card--not-installed"]
                        // },

                        #[watch]
                        set_filename: Some(model.variant.get_image()),

                        set_content_fit: gtk::ContentFit::Cover
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => GameCardComponentInput::EmitCardClick

                        // #[watch]
                        // set_icon_name: if model.installed {
                        //     "media-playback-start-symbolic"
                        // } else {
                        //     "folder-download-symbolic"
                        // }
                    }
                },

                gtk::Label {
                    set_margin_all: 12,

                    #[watch]
                    set_visible: model.display_title,

                    #[watch]
                    set_label: model.variant.get_title()
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
            width: 260,
            height: 364, // 10:14
            variant: init,
            installed: true,
            clickable: true,
            display_title: true
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameCardComponentInput::SetVariant(variant) => self.variant = variant,
            GameCardComponentInput::SetWidth(width) => self.width = width,
            GameCardComponentInput::SetHeight(height) => self.height = height,
            GameCardComponentInput::SetInstalled(installed) => self.installed = installed,
            GameCardComponentInput::SetClickable(clickable) => self.clickable = clickable,
            GameCardComponentInput::SetDisplayTitle(display_title) => self.display_title = display_title,

            GameCardComponentInput::EmitCardClick => {
                sender.output(GameCardComponentOutput::CardClicked {
                    variant: self.variant,
                    installed: self.installed
                }).unwrap()
            }
        }
    }
}

#[derive(Debug)]
pub struct GameCardFactory {
    pub component: AsyncController<GameCardComponent>
}

#[relm4::factory(pub)]
impl FactoryComponent for GameCardFactory {
    type Init = GameVariant;
    type Input = GameCardComponentInput;
    type Output = GameCardComponentOutput;
    type CommandOutput = ();
    type ParentInput = MainAppMsg;
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            self.component.widget(),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<MainAppMsg> {
        match output {
            GameCardComponentOutput::CardClicked { variant, installed }
                => Some(MainAppMsg::OpenDetails { variant, installed })
        }
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        Self {
            component: GameCardComponent::builder()
                .launch(init)
                .forward(sender.output_sender(), std::convert::identity)
        }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
