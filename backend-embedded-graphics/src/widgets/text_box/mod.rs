use core::cell::Cell;

use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoFont, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{PixelColor, Rgb888},
    prelude::{Dimensions, Point, Size},
    primitives::Rectangle,
    text::renderer::{CharacterStyle, TextRenderer},
    Drawable,
};
use embedded_gui::{
    geometry::{measurement::MeasureSpec, MeasuredSize, Position},
    input::event::{Key, Modifier, ToStr},
    state::selection::Selected,
    widgets::text_box::{TextBox, TextBoxProperties},
    WidgetRenderer,
};
use embedded_text::{
    style::{HeightMode, TextBoxStyleBuilder, VerticalOverdraw},
    TextBox as EgTextBox,
};

pub use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};
use heapless::String;
use object_chain::Chain;

use crate::{widgets::text_box::plugin::Cursor, EgCanvas, ToPoint, ToRectangle};

mod plugin;

pub struct TextBoxStyle<T>
where
    T: TextRenderer + CharacterStyle<Color = <T as TextRenderer>::Color>,
{
    renderer: T,
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
    cursor: Cell<Cursor>,
    cursor_color: Option<<T as TextRenderer>::Color>,
}

impl<'a, C> TextBoxStyle<MonoTextStyle<'a, C>>
where
    C: PixelColor,
{
    pub fn get_text_color(&self) -> Option<C> {
        self.renderer.text_color
    }

    /// Customize the text color
    pub fn text_color(&mut self, text_color: C) {
        self.renderer = MonoTextStyleBuilder::from(&self.renderer)
            .text_color(text_color)
            .build();

        if self.cursor_color.is_none() {
            self.cursor_color(text_color);
        }
    }

    /// Customize the cursor color
    pub fn cursor_color(&mut self, color: C) {
        self.cursor_color = Some(color);
    }

    /// Customize the font
    pub fn font<'a2>(self, font: &'a2 MonoFont<'a2>) -> TextBoxStyle<MonoTextStyle<'a2, C>> {
        TextBoxStyle {
            renderer: MonoTextStyleBuilder::from(&self.renderer)
                .font(font)
                .build(),
            horizontal: self.horizontal,
            vertical: self.vertical,
            cursor: Cell::new(Cursor::default()),
            cursor_color: None,
        }
    }
}

impl<F, C> TextBoxProperties for TextBoxStyle<F>
where
    F: TextRenderer<Color = C> + CharacterStyle<Color = C>,
    C: PixelColor + From<Rgb888>,
{
    fn measure_text(&self, text: &str, spec: MeasureSpec) -> MeasuredSize {
        let max_width = spec.width.largest().unwrap_or(u32::MAX);
        let max_height = spec.height.largest().unwrap_or(u32::MAX);

        if spec.height.is_exact() {
            return MeasuredSize {
                width: max_width,
                height: spec.height.largest().unwrap(),
            };
        }

        let bounding_box = EgTextBox::with_textbox_style(
            text,
            Rectangle::new(Point::zero(), Size::new(max_width, max_height)),
            self.renderer.clone(),
            TextBoxStyleBuilder::new()
                .height_mode(HeightMode::ShrinkToText(VerticalOverdraw::Hidden))
                .build(),
        )
        .fit_height()
        .bounding_box();

        MeasuredSize {
            width: bounding_box.size.width,
            height: bounding_box.size.height,
        }
    }

    fn handle_keypress<const N: usize>(
        &mut self,
        key: Key,
        modifier: Modifier,
        text: &mut String<N>,
    ) {
        let cursor = self.cursor.get_mut();

        match key {
            Key::ArrowUp => cursor.cursor_up(),
            Key::ArrowDown => cursor.cursor_down(),
            Key::ArrowLeft => cursor.cursor_left(),
            Key::ArrowRight => cursor.cursor_right(),
            Key::Del => cursor.delete_after(text),
            Key::Backspace => cursor.delete_before(text),
            _ => {
                if let Some(str) = (key, modifier).to_str() {
                    cursor.insert(text, str);
                }
            }
        }
    }

    fn handle_cursor_down(&mut self, coordinates: Position) {
        self.cursor.get_mut().move_cursor_to(coordinates.to_point())
    }
}

pub trait TextBoxStyling<'a, C, T, const N: usize>: Sized
where
    C: PixelColor,
    T: TextRenderer + CharacterStyle<Color = <T as TextRenderer>::Color>,
{
    type Color;

    fn text_color(mut self, color: Self::Color) -> Self {
        self.set_text_color(color);
        self
    }

    fn set_text_color(&mut self, color: Self::Color);

    fn text_renderer<T2>(self, renderer: T2) -> TextBox<TextBoxStyle<T2>, N>
    where
        T2: TextRenderer + CharacterStyle<Color = <T2 as TextRenderer>::Color>,
        <T2 as TextRenderer>::Color: From<Rgb888>;

    fn style<P>(self, props: P) -> TextBox<P, N>
    where
        P: TextBoxProperties;

    fn horizontal_alignment(self, alignment: HorizontalAlignment) -> Self;

    fn vertical_alignment(self, alignment: VerticalAlignment) -> Self;

    fn cursor_color(self, color: Self::Color) -> Self;
}

impl<'a, C, const N: usize> TextBoxStyling<'a, C, MonoTextStyle<'a, C>, N>
    for TextBox<TextBoxStyle<MonoTextStyle<'a, C>>, N>
where
    C: PixelColor + From<Rgb888>,
{
    type Color = C;

    fn set_text_color(&mut self, color: Self::Color) {
        self.label_properties.text_color(color);
    }

    fn text_renderer<T>(self, renderer: T) -> TextBox<TextBoxStyle<T>, N>
    where
        T: TextRenderer + CharacterStyle<Color = <T as TextRenderer>::Color>,
        <T as TextRenderer>::Color: From<Rgb888>,
    {
        let horizontal = self.label_properties.horizontal;
        let vertical = self.label_properties.vertical;
        let cursor = self.label_properties.cursor.clone();

        self.style(TextBoxStyle {
            renderer,
            horizontal,
            vertical,
            cursor,
            cursor_color: None, // TODO: convert
        })
    }

    fn style<P>(self, props: P) -> TextBox<P, N>
    where
        P: TextBoxProperties,
    {
        TextBox {
            state: self.state,
            parent_index: self.parent_index,
            text: self.text,
            bounds: self.bounds,
            label_properties: props,
        }
    }

    fn horizontal_alignment(self, alignment: HorizontalAlignment) -> Self {
        let renderer = self.label_properties.renderer;
        let horizontal = alignment;
        let vertical = self.label_properties.vertical;
        let cursor = self.label_properties.cursor.clone();
        let cursor_color = self.label_properties.cursor_color;

        self.style(TextBoxStyle {
            renderer,
            horizontal,
            vertical,
            cursor,
            cursor_color,
        })
    }

    fn vertical_alignment(self, alignment: VerticalAlignment) -> Self {
        let renderer = self.label_properties.renderer;
        let horizontal = self.label_properties.horizontal;
        let vertical = alignment;
        let cursor = self.label_properties.cursor.clone();
        let cursor_color = self.label_properties.cursor_color;

        self.style(TextBoxStyle {
            renderer,
            horizontal,
            vertical,
            cursor,
            cursor_color,
        })
    }

    fn cursor_color(self, color: C) -> Self {
        let renderer = self.label_properties.renderer;
        let horizontal = self.label_properties.horizontal;
        let vertical = self.label_properties.vertical;
        let cursor = self.label_properties.cursor.clone();
        let cursor_color = Some(color);

        self.style(TextBoxStyle {
            renderer,
            horizontal,
            vertical,
            cursor,
            cursor_color,
        })
    }
}

/// Font settings specific to `MonoFont`'s renderer.
pub trait MonoFontTextBoxStyling<C, const N: usize>: Sized
where
    C: PixelColor,
{
    fn font<'a>(self, font: &'a MonoFont<'a>) -> TextBox<TextBoxStyle<MonoTextStyle<'a, C>>, N>;
}

impl<'a, C, const N: usize> MonoFontTextBoxStyling<C, N>
    for TextBox<TextBoxStyle<MonoTextStyle<'a, C>>, N>
where
    C: PixelColor + From<Rgb888>,
{
    fn font<'a2>(
        self,
        font: &'a2 MonoFont<'a2>,
    ) -> TextBox<TextBoxStyle<MonoTextStyle<'a2, C>>, N> {
        let renderer = MonoTextStyleBuilder::from(&self.label_properties.renderer)
            .font(font)
            .build();
        let horizontal = self.label_properties.horizontal;
        let vertical = self.label_properties.vertical;
        let cursor = self.label_properties.cursor.clone();
        let cursor_color = self.label_properties.cursor_color;

        self.style(TextBoxStyle {
            renderer,
            horizontal,
            vertical,
            cursor,
            cursor_color,
        })
    }
}

impl<F, C, DT, const N: usize> WidgetRenderer<EgCanvas<DT>> for TextBox<TextBoxStyle<F>, N>
where
    F: TextRenderer<Color = C> + CharacterStyle<Color = C>,
    C: PixelColor + From<Rgb888>,
    DT: DrawTarget<Color = C>,
{
    fn draw(&self, canvas: &mut EgCanvas<DT>) -> Result<(), DT::Error> {
        let cursor_color = self.label_properties.cursor_color;
        if self.state.has_state(Selected) && cursor_color.is_some() {
            let textbox = EgTextBox::with_textbox_style(
                self.text.as_ref(),
                self.bounds.to_rectangle(),
                self.label_properties.renderer.clone(),
                TextBoxStyleBuilder::new()
                    .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
                    .alignment(self.label_properties.horizontal)
                    .vertical_alignment(self.label_properties.vertical)
                    .build(),
            )
            .add_plugin(
                self.label_properties
                    .cursor
                    .get()
                    .plugin(cursor_color.unwrap()),
            );

            let result = textbox.draw(&mut canvas.target).map(|_| ());

            let Chain { object: plugin } = textbox.take_plugins();
            self.label_properties.cursor.set(plugin.get_cursor());

            result
        } else {
            EgTextBox::with_textbox_style(
                self.text.as_ref(),
                self.bounds.to_rectangle(),
                self.label_properties.renderer.clone(),
                TextBoxStyleBuilder::new()
                    .height_mode(HeightMode::Exact(VerticalOverdraw::Hidden))
                    .alignment(self.label_properties.horizontal)
                    .vertical_alignment(self.label_properties.vertical)
                    .build(),
            )
            .draw(&mut canvas.target)
            .map(|_| ())
        }
    }
}

macro_rules! textbox_for_charset {
    ($charset:ident, $font:ident) => {
        pub mod $charset {
            use core::cell::Cell;
            use embedded_graphics::{
                mono_font::{$charset, MonoTextStyle},
                pixelcolor::PixelColor,
            };
            use embedded_gui::{
                geometry::BoundingBox, state::WidgetState, widgets::text_box::TextBox,
            };
            use embedded_text::alignment::{HorizontalAlignment, VerticalAlignment};

            use crate::{
                themes::Theme,
                widgets::text_box::{plugin::Cursor, TextBoxStyle},
            };

            pub trait TextBoxConstructor<'a, C, const N: usize>
            where
                C: PixelColor,
            {
                fn new(text: heapless::String<N>)
                    -> TextBox<TextBoxStyle<MonoTextStyle<'a, C>>, N>;
            }

            impl<'a, 'b, 'c, C, const N: usize> TextBoxConstructor<'a, C, N>
                for TextBox<TextBoxStyle<MonoTextStyle<'a, C>>, N>
            where
                C: PixelColor + Theme,
            {
                fn new(text: heapless::String<N>) -> Self {
                    TextBox {
                        state: WidgetState::default(),
                        parent_index: 0,
                        text,
                        label_properties: TextBoxStyle {
                            renderer: MonoTextStyle::new(
                                &$charset::$font,
                                <C as Theme>::TEXT_COLOR,
                            ),
                            horizontal: HorizontalAlignment::Left,
                            vertical: VerticalAlignment::Top,
                            cursor: Cell::new(Cursor::default()),
                            cursor_color: Some(<C as Theme>::TEXT_COLOR),
                        },
                        bounds: BoundingBox::default(),
                    }
                }
            }
        }
    };

    ($charset:ident) => {
        textbox_for_charset!($charset, FONT_6X10);
    };
}

textbox_for_charset!(ascii);
textbox_for_charset!(iso_8859_1);
textbox_for_charset!(iso_8859_10);
textbox_for_charset!(iso_8859_13);
textbox_for_charset!(iso_8859_14);
textbox_for_charset!(iso_8859_15);
textbox_for_charset!(iso_8859_16);
textbox_for_charset!(iso_8859_2);
textbox_for_charset!(iso_8859_3);
textbox_for_charset!(iso_8859_4);
textbox_for_charset!(iso_8859_5);
textbox_for_charset!(iso_8859_7);
textbox_for_charset!(iso_8859_9);
textbox_for_charset!(jis_x0201, FONT_6X13);
