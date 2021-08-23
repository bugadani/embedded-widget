//! Light theme for toggle buttons.

use crate::toggle_button_style_rgb;

pub mod binary_color {
    use crate::toggle_button_style;
    use embedded_graphics::{
        mono_font::{ascii::FONT_6X10, MonoFont},
        pixelcolor::BinaryColor,
    };

    toggle_button_style!(
        ToggleButton<BinaryColor, FONT_6X10> {
            Unchecked {
                Inactive {
                    label: Off,
                    border: On,
                    background: On,
                },
                Idle {
                    label: Off,
                    border: On,
                    background: On,
                },
                Hovered {
                    label: On,
                    border: On,
                    background: Off,
                },
                Pressed {
                    label: Off,
                    border: On,
                    background: On,
                }
            },
            Checked {
                Inactive {
                    label: Off,
                    border: On,
                    background: On,
                },
                Idle {
                    label: Off,
                    border: On,
                    background: On,
                },
                Hovered {
                    label: On,
                    border: On,
                    background: Off,
                },
                Pressed {
                    label: Off,
                    border: On,
                    background: On,
                }
            }
        }
    );
}

toggle_button_style_rgb!(
    ToggleButton<FONT_6X10> {
        Unchecked {
            Inactive {
                label: CSS_LIGHT_GRAY,
                border: CSS_DIM_GRAY,
                background: CSS_DIM_GRAY,
            },
            Idle {
                label: WHITE,
                border: CSS_STEEL_BLUE,
                background: CSS_STEEL_BLUE,
            },
            Hovered {
                label: WHITE,
                border: CSS_DODGER_BLUE,
                background: CSS_DODGER_BLUE,
            },
            Pressed {
                label: WHITE,
                border: CSS_LIGHT_STEEL_BLUE,
                background: CSS_LIGHT_STEEL_BLUE,
            }
        },
        Checked {
            Inactive {
                label: CSS_LIGHT_GRAY,
                border: CSS_DIM_GRAY,
                background: CSS_DIM_GRAY,
            },
            Idle {
                label: WHITE,
                border: CSS_STEEL_BLUE,
                background: CSS_STEEL_BLUE,
            },
            Hovered {
                label: WHITE,
                border: CSS_DODGER_BLUE,
                background: CSS_DODGER_BLUE,
            },
            Pressed {
                label: WHITE,
                border: CSS_LIGHT_STEEL_BLUE,
                background: CSS_LIGHT_STEEL_BLUE,
            }
        }
    }
);
