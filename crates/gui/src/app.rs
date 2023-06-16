use std::sync::{Arc, RwLock};

use egui_dock::{DockArea, NodeIndex, Style, Tree};

use crate::shared_state::SharedState;

mod about;
mod connection;
mod filter;
mod hex_viewer;
mod packet_list;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, shared_state: &mut SharedState);
}

/// Something to view
pub trait Window: View {
    fn new() -> Self
    where
        Self: Sized;

    /// `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &'static str;

    fn default_width(&self) -> f32 {
        320.0
    }

    /// Show windows, etc
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        shared_state: Arc<RwLock<SharedState>>,
    ) {
        let mut state = shared_state.write().unwrap();
        egui::Window::new(self.name())
            .default_width(self.default_width())
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui, &mut state);
            });
    }
}

struct TabViewer {
    shared_state: Arc<RwLock<SharedState>>,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Box<dyn Window>;

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
    tree: Tree<Box<dyn Window>>,
    shared_state: Arc<RwLock<SharedState>>,
    tab_viewer: TabViewer,
}

impl GuiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut tree: Tree<Box<dyn Window>> =
            Tree::new(vec![Box::new(connection::Connection::new())]);
        let [a, b] = tree.split_right(
            NodeIndex::root(),
            0.3,
            vec![Box::new(packet_list::PacketList::new())],
        );

        // split a down with filters
        let [_, _] = tree.split_below(a, 0.25, vec![Box::new(filter::Filter::new())]);
        let [_, _] = tree.split_below(b, 0.5, vec![Box::new(hex_viewer::HexView::new())]);

        let mut shared_state = SharedState::default();

        if let Some(storage) = cc.storage {
            if let Some(value) = eframe::get_value::<SharedState>(storage, eframe::APP_KEY) {
                shared_state = value.merge(shared_state);
            }
        }

        let shared_state = Arc::new(RwLock::new(shared_state));

        let event_shared_state = shared_state.clone();
        tokio::spawn(async move {
            let receiver = event_shared_state.write().unwrap().receiver.take().unwrap();
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    // Todo: handle events here
                }
            }
        });

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
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut self.tab_viewer);
    }
}
