use std::sync::{Arc, RwLock};

use egui_dock::{DockArea, NodeIndex, Style, Tree};

use crate::shared_state::{Event, SharedState};

mod about;
mod connection;
mod filter;
mod hex_viewer;
mod packet_list;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, shared_state: &mut SharedState);
}

/// Something to view
pub trait Tab: View {
    fn new() -> Self
    where
        Self: Sized;

    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;
}

struct TabViewer {
    shared_state: Arc<RwLock<SharedState>>,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Box<dyn Tab>;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui, &mut self.shared_state.write().unwrap());
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}

pub struct GuiApp {
    tree: Tree<Box<dyn Tab>>,
    shared_state: Arc<RwLock<SharedState>>,
    tab_viewer: TabViewer,
}

impl GuiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Default Application Layout
        let mut tree: Tree<Box<dyn Tab>> = Tree::new(vec![Box::new(connection::Connection::new())]);

        let [a, b] = tree.split_right(
            NodeIndex::root(),
            0.3,
            vec![Box::new(packet_list::PacketList::new())],
        );

        let [_, _] = tree.split_below(a, 0.25, vec![Box::new(filter::Filter::new())]);
        let [_, _] = tree.split_below(b, 0.5, vec![Box::new(hex_viewer::HexView::new())]);

        // Persistant Storage
        let mut shared_state = SharedState::default();

        if let Some(storage) = cc.storage {
            if let Some(value) = eframe::get_value::<SharedState>(storage, eframe::APP_KEY) {
                shared_state = value.merge(shared_state);
            }
        }

        let shared_state = Arc::new(RwLock::new(shared_state));

        // Event Handling
        let event_shared_state = shared_state.clone();
        tokio::spawn(async move {
            let receiver = event_shared_state.write().unwrap().receiver.take().unwrap();
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    Event::StartListening => {
                        let mut state = event_shared_state.write().unwrap();
                        if state.is_listening {
                            continue;
                        }
                        state.is_listening = true;
                    }
                    Event::StopListening => {
                        let mut state = event_shared_state.write().unwrap();
                        if !state.is_listening {
                            continue;
                        }
                        state.is_listening = false;
                    }
                }
            }
        });

        // Tab Viewer
        let tab_viewer = TabViewer {
            shared_state: shared_state.clone(),
        };

        Self {
            shared_state,
            tree,
            tab_viewer,
        }
    }
}

impl eframe::App for GuiApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            eframe::APP_KEY,
            &*self.shared_state.read().unwrap(),
        );
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.tree)
            .show_add_buttons(false)
            .show_add_popup(false)
            .show_close_buttons(false)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);
    }
}
