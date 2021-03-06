use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use kakapo::app;
use kakapo::view::{UserData, UserDataMut, View, ViewRefs, WidgetCache, WidgetTree};
use kakapo::view_model::ViewModel;
use kakapo::widgets;
use kakapo::widgets::ButtonDelegate;

struct SharedState {
    view_refs: ViewRefs,
    first: AtomicBool,
    second: AtomicBool,
}

struct AppData {
    two_buttons: bool,
    shared_state: Arc<SharedState>,
}

impl AppData {
    fn new() -> AppData {
        AppData {
            two_buttons: false,
            shared_state: Arc::new(SharedState {
                view_refs: ViewRefs::new(),
                first: AtomicBool::new(false),
                second: AtomicBool::new(false),
            }),
        }
    }
}

impl ViewModel for AppData {
    fn view_refs(&self) -> &ViewRefs {
        &self.shared_state.view_refs
    }
}

#[derive(Copy, Clone)]
struct PrimaryButtonDelegate;

impl ButtonDelegate for PrimaryButtonDelegate {
    fn pressed(&mut self, parent: UserDataMut<'_>) {
        let app_data = parent.unwrap().downcast_mut::<AppData>().unwrap();
        app_data.two_buttons = !app_data.two_buttons;
        app_data.shared_state.view_refs.update();
        println!("First");
    }
}

#[derive(Copy, Clone)]
struct SecondaryButtonDelegate(bool);

impl ButtonDelegate for SecondaryButtonDelegate {
    fn pressed(&mut self, parent: UserDataMut<'_>) {
        println!("Second");
        let app_data = parent.unwrap().downcast_mut::<AppData>().unwrap();
        let first = self.0;
        let state = Arc::clone(&app_data.shared_state);
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            let field = if first { &state.first } else { &state.second };
            field.fetch_xor(true, Ordering::SeqCst);
            state.view_refs.update();
        });
    }
}

fn secondary_button_colour(data: &AtomicBool) -> [f32; 4] {
    if data.load(Ordering::SeqCst) {
        [0.0, 1.0, 1.0, 1.0]
    } else {
        [0.0, 0.0, 1.0, 1.0]
    }
}

struct AppView {}

impl View for AppView {
    fn view(&mut self, cache: &mut WidgetCache, user_data: UserData<'_>) -> WidgetTree {
        println!("View!");
        let data = user_data.unwrap().downcast_ref::<AppData>().unwrap();
        let mut b = widgets::Box::new()
            .append(
                widgets::Button::new(
                    if data.two_buttons {
                        [0.0, 1.0, 0.0, 1.0]
                    } else {
                        [1.0, 0.0, 0.0, 1.0]
                    },
                    PrimaryButtonDelegate,
                )
                .add_text(&wgpu_glyph::Text::new("Primary")),
            )
            .append(
                widgets::Button::new(
                    secondary_button_colour(&data.shared_state.first),
                    SecondaryButtonDelegate(true),
                )
                .add_text(&wgpu_glyph::Text::new("A")),
            );
        if data.two_buttons {
            b = b.append(
                widgets::Button::new(
                    secondary_button_colour(&data.shared_state.second),
                    SecondaryButtonDelegate(false),
                )
                .add_text(&wgpu_glyph::Text::new("B")),
            );
        }
        cache.build(b)
    }
}

fn main() {
    let mut app = app::AppBuilder::new();
    app.add_window(AppView {}, AppData::new());
    app.run()
}
