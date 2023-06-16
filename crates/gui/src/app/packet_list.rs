use eframe::epaint::{PathShape, RectShape};
use egui::{
    Color32, Pos2, Rect, Response, Rgba, Rounding, Sense, Shape, Stroke, TextStyle, Ui, Vec2,
    WidgetText,
};

use proxy_lib::{Packet, PacketSide};

use super::{SharedState, Tab, View};

pub struct PacketList {}

impl Tab for PacketList {
    fn new() -> Self {
        Self {}
    }

    fn name(&self) -> &'static str {
        "Packets"
    }
}

impl View for PacketList {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut SharedState) {
        if ui.button("Clear").clicked() {
            state.selected_packet = None;
            state.packets.write().unwrap().clear();
        }

        let packets = state.packets.read().unwrap();
        for (i, packet) in packets.iter().enumerate() {
            if let Some(filtered) = state.packet_filter.get(packet) {
                if !filtered {
                    continue;
                }
            }

            let selected = {
                if let Some(selected) = state.selected_packet {
                    selected == i
                } else {
                    false
                }
            };

            if draw_packet_widget(ui, packet, selected).clicked() {
                state.selected_packet = Some(i);
            }
        }
    }
}

fn draw_packet_widget(ui: &mut Ui, packet: &Packet, selected: bool) -> Response {
    let (mut rect, response) = ui.allocate_at_least(
        Vec2 {
            x: ui.available_width(),
            y: 24.0,
        },
        Sense::click(),
    );

    let fill = match selected /*packet.selected*/ {
        true => Rgba::from_rgba_premultiplied(0.3, 0.3, 0.3, 0.4),
        false => Rgba::from_rgba_premultiplied(0.0, 0.0, 0.0, 0.0),
    };

    let text_color: Color32 = match selected /*packet.selected*/ {
        true => Rgba::from_rgba_premultiplied(0.0, 0.0, 0.0, 1.0).into(),
        false => ui.visuals().strong_text_color(),
    };

    if ui.is_rect_visible(rect) {
        ui.painter().add(Shape::Rect(RectShape {
            rect,
            rounding: Rounding::none(),
            fill: fill.into(),
            stroke: Stroke::new(1.0, Rgba::BLACK),
        }));

        let shape = get_triangle(packet.side, &rect);
        ui.painter().add(Shape::Path(shape));

        let identifier: WidgetText = format!("0x{:0>2X?}", packet.id).into();

        let identifier =
            identifier.into_galley(ui, Some(false), rect.width() - 21.0, TextStyle::Button);

        let label: WidgetText = packet.name.into();
        let label = label.into_galley(ui, Some(false), rect.width() - 60.0, TextStyle::Button);

        let timestamp: WidgetText = systemtime_strftime(packet.timestamp.unwrap()).into();
        let timestamp =
            timestamp.into_galley(ui, Some(false), rect.width() - 60.0, TextStyle::Button);

        identifier.paint_with_fallback_color(
            ui.painter(),
            Pos2 {
                x: rect.left() + 21.0,
                y: rect.top() + 6.0,
            },
            ui.visuals().weak_text_color(),
        );

        rect.set_width(rect.width() - 5.0);

        let label_width = label.size().x + 50.0;

        label.paint_with_fallback_color(
            &ui.painter().with_clip_rect(rect),
            Pos2 {
                x: rect.left() + 55.0,
                y: rect.top() + 6.0,
            },
            text_color,
        );

        timestamp.paint_with_fallback_color(
            &ui.painter().with_clip_rect(rect),
            Pos2 {
                x: rect.left() + label_width + 8.0,
                y: rect.top() + 6.0,
            },
            ui.visuals().weak_text_color(),
        );
    }

    response
}

fn get_triangle(direction: PacketSide, outer_rect: &Rect) -> PathShape {
    let rect = Rect::from_min_size(
        Pos2 {
            x: outer_rect.left() + 6.0,
            y: outer_rect.top() + 8.0,
        },
        Vec2 { x: 8.0, y: 8.0 },
    );

    let color = match direction {
        PacketSide::Clientbound => Rgba::from_rgb(255.0, 0.0, 0.0),
        PacketSide::Serverbound => Rgba::from_rgb(0.0, 255.0, 0.0),
    };

    let points = match direction {
        PacketSide::Clientbound => vec![
            Pos2 {
                x: rect.left() + (rect.width() / 2.0),
                y: rect.top() + rect.height(),
            },
            Pos2 {
                x: rect.left() + 0.0,
                y: rect.top(),
            },
            Pos2 {
                x: rect.left() + rect.width(),
                y: rect.top(),
            },
        ],
        PacketSide::Serverbound => vec![
            Pos2 {
                x: rect.left() + (rect.width() / 2.0),
                y: rect.top() + 0.0,
            },
            Pos2 {
                x: rect.left() + 0.0,
                y: rect.top() + rect.height(),
            },
            Pos2 {
                x: rect.left() + rect.width(),
                y: rect.top() + rect.height(),
            },
        ],
    };

    let mut shape = PathShape::closed_line(points, Stroke::new(2.0, color));
    shape.fill = color.into();

    shape
}

pub fn systemtime_strftime(odt: time::OffsetDateTime) -> String {
    let hour = odt.hour();
    let minute = odt.minute();
    let second = odt.second();
    let millis = odt.millisecond();

    format!("{hour:0>2}:{minute:0>2}:{second:0>2}.{millis:0>4}")
}
