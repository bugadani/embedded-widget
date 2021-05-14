use std::{thread, time::Duration};

use backend_embedded_graphics::{
    themes::Theme,
    widgets::{
        label::ascii::LabelConstructor,
        primitives::{background::BackgroundStyle, border::BorderStyle},
    },
    EgCanvas,
};
use embedded_graphics::{
    draw_target::DrawTarget, pixelcolor::BinaryColor, prelude::Size as EgSize,
};
use embedded_graphics_simulator::{
    sdl2::MouseButton, BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent,
    Window as SimWindow,
};
use embedded_gui::{
    data::BoundData,
    geometry::Position,
    input::event::{InputEvent, PointerEvent},
    state::WidgetState,
    widgets::{
        button::Button,
        label::Label,
        layouts::linear::{column::Column, row::Row},
        primitives::{
            background::Background,
            border::Border,
            fill::{Center, FillParent, HorizontalAndVertical},
            spacing::Spacing,
        },
        Widget,
    },
    Window,
};

fn convert_input(event: SimulatorEvent) -> Result<InputEvent, bool> {
    unsafe {
        // This is fine for a demo
        static mut MOUSE_DOWN: bool = false;
        match event {
            SimulatorEvent::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                point,
            } => {
                MOUSE_DOWN = false;
                Ok(InputEvent::PointerEvent(
                    Position {
                        x: point.x,
                        y: point.y,
                    },
                    PointerEvent::Up,
                ))
            }
            SimulatorEvent::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                point,
            } => {
                MOUSE_DOWN = true;
                Ok(InputEvent::PointerEvent(
                    Position {
                        x: point.x,
                        y: point.y,
                    },
                    PointerEvent::Down,
                ))
            }
            SimulatorEvent::MouseMove { point } => Ok(InputEvent::PointerEvent(
                Position {
                    x: point.x,
                    y: point.y,
                },
                if MOUSE_DOWN {
                    PointerEvent::Drag
                } else {
                    PointerEvent::Hover
                },
            )),
            SimulatorEvent::Quit => Err(true),
            _ => Err(false),
        }
    }
}

fn update_button_background<W: Widget>(
    widget: &mut Background<W, BackgroundStyle<BinaryColor>>,
    state: WidgetState,
) {
    if state.has_state(Button::STATE_HOVERED) {
        widget.set_background_color(BinaryColor::Off)
    } else if state.has_state(Button::STATE_PRESSED) {
        widget.set_background_color(BinaryColor::On)
    } else {
        widget.set_background_color(BinaryColor::Off)
    };
}

fn update_button_border<W: Widget>(
    widget: &mut Border<W, BorderStyle<BinaryColor>>,
    state: WidgetState,
) {
    if state.has_state(Button::STATE_HOVERED) {
        widget.set_border_color(BinaryColor::On)
    } else if state.has_state(Button::STATE_PRESSED) {
        widget.set_border_color(BinaryColor::Off)
    } else {
        widget.set_border_color(BinaryColor::Off)
    };
}

// While this return type is ugly as, it can be generated by the compiler
// (`-> _` gives the type as a compile error ❤)
fn button_with_style<W: Widget>(
    inner: W,
) -> Button<
    Background<
        Border<FillParent<W, HorizontalAndVertical, Center, Center>, BorderStyle<BinaryColor>>,
        BackgroundStyle<BinaryColor>,
    >,
> {
    Button::new(
        Background::new(
            Border::new(
                FillParent::both(inner)
                    .align_horizontal(Center)
                    .align_vertical(Center),
            )
            .border_color(BinaryColor::Off)
            .on_state_changed(update_button_border),
        )
        .background_color(BinaryColor::Off)
        .on_state_changed(update_button_background),
    )
}

fn main() {
    let display = SimulatorDisplay::new(EgSize::new(128, 64));

    let flag = BoundData::new(true, |data| {
        println!("Data changed to {:?}", data);
    });

    let mut gui = Window::new(
        EgCanvas::new(display),
        Column::new()
            .add(
                Row::new()
                    .add(FillParent::horizontal(Label::new("Hello,")).align_horizontal(Center))
                    .weight(1)
                    .add(FillParent::horizontal(Label::new("World!")).align_horizontal(Center))
                    .weight(1),
            )
            .add(
                Spacing::new(
                    button_with_style(Label::new("Click me").bind(&flag).on_data_changed(
                        |widget, data| {
                            widget.text = if *data { "on" } else { "off" };
                        },
                    ))
                    .bind(&flag)
                    .on_clicked(|data| {
                        *data = !*data;
                        println!("Clicked!");
                    }),
                )
                .all(4),
            ),
    );

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    let mut window = SimWindow::new("GUI demonstration", &output_settings);

    loop {
        gui.canvas
            .target
            .clear(BinaryColor::BACKGROUND_COLOR)
            .unwrap();

        gui.update();
        gui.measure();
        gui.arrange();
        gui.draw().unwrap();

        // Update the window.
        window.update(&gui.canvas.target);

        // Handle key and mouse events.
        for event in window.events() {
            match convert_input(event) {
                Ok(input) => {
                    gui.input_event(input);
                }
                Err(true) => return,
                _ => {}
            }
        }

        // Wait for a little while.
        thread::sleep(Duration::from_millis(10));
    }
}
