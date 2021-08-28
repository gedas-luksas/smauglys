use std::{
    cell::RefCell,
    sync::mpsc::{channel, Receiver, TryRecvError},
};

use log::{debug, error, trace};
use nwd::NwgUi;
use nwg::NativeUi;

use crate::error::IResult;

pub(crate) enum Message {
    ProgressUpdate { progress: u32, details: String },
    Finished,
    Abort(String),
}

#[derive(NwgUi)]
pub struct InstallerApp {
    #[nwg_control(size: (530, 300), position: (300, 300), title: "Smauglys: diegimo programa", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [InstallerApp::exit], OnInit: [InstallerApp::show_initial_wiew], OnMinMaxInfo: [InstallerApp::set_resize(SELF, EVT_DATA)] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1)]
    grid_initial: nwg::GridLayout,

    #[nwg_control(text: "Diegimo programa.", flags: "VISIBLE|MULTI_LINE")]
    #[nwg_layout_item(layout: grid_initial, row: 0, col: 0, row_span: 4)]
    explanation: nwg::RichLabel,

    #[nwg_control(text: "Sutinku su Python licencija.")]
    #[nwg_layout_item(layout: grid_initial, row: 5, col: 0)]
    #[nwg_events( OnButtonClick: [InstallerApp::python_license_checkbox_click] )]
    python_license_checkbox: nwg::CheckBox,

    #[nwg_control(text: "Sutinku su Microsoft VS Code licencija.")]
    #[nwg_layout_item(layout: grid_initial, row: 6, col: 0)]
    #[nwg_events( OnButtonClick: [InstallerApp::vscode_license_checkbox_click] )]
    vscode_license_checkbox: nwg::CheckBox,

    #[nwg_control(text: "Įdiegti")]
    #[nwg_layout_item(layout: grid_initial, row: 7, col: 0)]
    #[nwg_events( OnButtonClick: [InstallerApp::show_progress_view] )]
    install_button: nwg::Button,

    #[nwg_layout(parent: window, spacing: 1)]
    grid_installing: nwg::GridLayout,

    #[nwg_control(parent: window)]
    #[nwg_events(OnNotice: [InstallerApp::update_progress_bar])]
    progress_bar_notice: nwg::Notice,

    #[nwg_control(step: 1, range: 0..3)]
    #[nwg_layout_item(layout: grid_installing, row: 1, col: 0)]
    progress_bar: nwg::ProgressBar,

    #[nwg_control(text: "Inicializuojama.", flags: "VISIBLE|MULTI_LINE")]
    #[nwg_layout_item(layout: grid_installing, row: 0, col: 0)]
    progress_bar_details: nwg::RichLabel,

    progress_bar_receiver: RefCell<Option<Receiver<Message>>>,

    #[nwg_layout(parent: window, spacing: 1)]
    grid_final: nwg::GridLayout,

    #[nwg_control(text: "Diegimas sėkmingai baigtas", flags: "VISIBLE|MULTI_LINE")]
    #[nwg_layout_item(layout: grid_initial, row: 0, col: 0, row_span: 4)]
    success_message: nwg::RichLabel,

    #[nwg_control(text: "Baigti")]
    #[nwg_layout_item(layout: grid_initial, row: 5, col: 0)]
    #[nwg_events( OnButtonClick: [InstallerApp::close] )]
    finish_button: nwg::Button,
}

impl InstallerApp {
    fn update_progress_bar(&self) {
        trace!("[enter] update_progress_bar");
        if let Some(receiver) = &*self.progress_bar_receiver.borrow() {
            match receiver.try_recv() {
                Ok(Message::ProgressUpdate { progress, details }) => {
                    self.progress_bar.set_pos(progress);
                    self.progress_bar_details.set_text(&details);
                }
                Ok(Message::Finished) => {
                    self.show_final_wiew();
                }
                Ok(Message::Abort(abort_message)) => {
                    error!("critical error: {}", abort_message);
                    nwg::modal_error_message(&self.window, "Kritinė klaida", &abort_message);
                    nwg::stop_thread_dispatch();
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Disconnected channel.");
                    self.show_final_wiew();
                }
                Err(TryRecvError::Empty) => {
                    debug!("empty channel");
                }
            }
        } else {
            error!("Internal error: progress_bar_receiver is None");
            nwg::modal_error_message(&self.window, "Kritinė klaida", "Vidinė klaida");
            nwg::stop_thread_dispatch();
        }
        trace!("[exit] update_progress_bar");
    }
    fn set_resize(&self, data: &nwg::EventData) {
        let data = data.on_min_max();
        data.set_min_size(200, 200);
    }
    fn exit(&self) {
        nwg::stop_thread_dispatch();
        error!("TODO: stop the spinning thread.");
        unimplemented!("TODO: Stop the spinning thread.")
    }
    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
    fn set_visible_initial_view(&self, visible: bool) {
        self.explanation.set_visible(visible);
        self.python_license_checkbox.set_visible(visible);
        self.vscode_license_checkbox.set_visible(visible);
        self.install_button.set_visible(visible);
    }
    fn set_visible_progress_view(&self, visible: bool) {
        self.progress_bar.set_visible(visible);
        self.progress_bar_details.set_visible(visible);
    }
    fn set_visible_final_wiew(&self, visible: bool) {
        self.success_message.set_visible(visible);
        self.finish_button.set_visible(visible);
    }
    fn show_initial_wiew(&self) {
        trace!("[enter] show_initial_wiew");
        self.set_visible_initial_view(true);
        self.set_visible_progress_view(false);
        self.set_visible_final_wiew(false);
        self.install_button.set_enabled(false);
        trace!("[exit] show_initial_wiew");
    }
    fn show_progress_view(&self) {
        trace!("[enter] show_progress_view");
        self.set_visible_initial_view(false);
        self.set_visible_progress_view(true);
        self.set_visible_final_wiew(false);
        let (sender, receiver) = channel();
        let mut receiver_borrow = self.progress_bar_receiver.borrow_mut();
        *receiver_borrow = Some(receiver);
        let sender_notice = self.progress_bar_notice.sender();
        std::thread::spawn(move || {
            crate::installation::install(sender_notice, sender);
        });
        trace!("[exit] show_progress_view");
    }
    fn show_final_wiew(&self) {
        trace!("[enter] show_final_wiew");
        self.set_visible_initial_view(false);
        self.set_visible_progress_view(false);
        self.set_visible_final_wiew(true);
        trace!("[exit] show_final_wiew");
    }
    fn python_license_checkbox_click(&self) {
        let state = (
            self.python_license_checkbox.check_state(),
            self.vscode_license_checkbox.check_state(),
        );
        match state {
            (nwg::CheckBoxState::Checked, nwg::CheckBoxState::Checked) => {
                self.install_button.set_enabled(true);
            }
            _ => {
                self.install_button.set_enabled(false);
            }
        }
    }
    fn vscode_license_checkbox_click(&self) {
        let state = (
            self.python_license_checkbox.check_state(),
            self.vscode_license_checkbox.check_state(),
        );
        match state {
            (nwg::CheckBoxState::Checked, nwg::CheckBoxState::Checked) => {
                self.install_button.set_enabled(true);
            }
            _ => {
                self.install_button.set_enabled(false);
            }
        }
    }
}

pub(crate) fn run() -> IResult {
    trace!("[enter] gui::run");
    nwg::init()?;
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .size(18)
        .family("Segoe UI")
        .build(&mut font)?;
    nwg::Font::set_global_default(Some(font));
    let initial_state = InstallerApp {
        progress_bar_receiver: RefCell::new(None),
        window: Default::default(),
        grid_initial: Default::default(),
        python_license_checkbox: Default::default(),
        vscode_license_checkbox: Default::default(),
        install_button: Default::default(),
        grid_installing: Default::default(),
        progress_bar_notice: Default::default(),
        progress_bar: Default::default(),
        progress_bar_details: Default::default(),
        grid_final: Default::default(),
        explanation: Default::default(),
        success_message: Default::default(),
        finish_button: Default::default(),
    };
    let _app = InstallerApp::build_ui(initial_state)?;
    nwg::dispatch_thread_events();
    trace!("[exit] gui::run");
    Ok(())
}