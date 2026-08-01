//! Geometry of the elements that make up the fancy tab bar.

use crate::units::PixelUnit;
use crate::Dimension;
use std::collections::BTreeMap;
use wezterm_dynamic::{FromDynamic, FromDynamicOptions, Object, ToDynamic, Value};

const fn px(v: f32) -> Dimension {
    Dimension::Pixels(v)
}
const fn cells(v: f32) -> Dimension {
    Dimension::Cells(v)
}

fn dim_field(
    obj: &Object,
    name: &str,
    options: FromDynamicOptions,
) -> Result<Dimension, wezterm_dynamic::Error> {
    match obj.get_by_str(name) {
        None | Some(Value::Null) => Ok(px(0.)),
        Some(v) => Ok(PixelUnit::from_dynamic(v, options)?.into()),
    }
}

/// Padding or margin. Either a single value for all four sides
/// (`'5px'`) or a table naming them (`{ left = '12px', top = '3px' }`).
/// In the table form a side that is left out is zero.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxDim {
    pub left: Dimension,
    pub right: Dimension,
    pub top: Dimension,
    pub bottom: Dimension,
}

impl BoxDim {
    pub const fn uniform(d: Dimension) -> Self {
        Self {
            left: d,
            right: d,
            top: d,
            bottom: d,
        }
    }

    pub const fn zero() -> Self {
        Self::uniform(px(0.))
    }

    pub const fn new(left: Dimension, right: Dimension, top: Dimension, bottom: Dimension) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

}

impl Default for BoxDim {
    fn default() -> Self {
        Self::zero()
    }
}

impl FromDynamic for BoxDim {
    fn from_dynamic(
        value: &Value,
        options: FromDynamicOptions,
    ) -> Result<Self, wezterm_dynamic::Error> {
        match value {
            Value::Object(obj) => Ok(Self {
                left: dim_field(obj, "left", options)?,
                right: dim_field(obj, "right", options)?,
                top: dim_field(obj, "top", options)?,
                bottom: dim_field(obj, "bottom", options)?,
            }),
            other => Ok(Self::uniform(PixelUnit::from_dynamic(other, options)?.into())),
        }
    }
}

impl ToDynamic for BoxDim {
    fn to_dynamic(&self) -> Value {
        let mut obj: Object = BTreeMap::new().into();
        obj.insert(Value::String("left".to_string()), self.left.to_dynamic());
        obj.insert(Value::String("right".to_string()), self.right.to_dynamic());
        obj.insert(Value::String("top".to_string()), self.top.to_dynamic());
        obj.insert(Value::String("bottom".to_string()), self.bottom.to_dynamic());
        Value::Object(obj)
    }
}

/// How an element is placed within the height available to it.
/// `Stretch` fills that height rather than picking a position in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementAlign {
    Top,
    Middle,
    Bottom,
    Stretch,
}

impl ElementAlign {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Middle => "middle",
            Self::Bottom => "bottom",
            Self::Stretch => "stretch",
        }
    }
}

impl FromDynamic for ElementAlign {
    fn from_dynamic(
        value: &Value,
        _options: FromDynamicOptions,
    ) -> Result<Self, wezterm_dynamic::Error> {
        match value {
            Value::String(s) => match s.to_ascii_lowercase().as_str() {
                "top" => Ok(Self::Top),
                "middle" | "center" | "centre" => Ok(Self::Middle),
                "bottom" => Ok(Self::Bottom),
                "stretch" | "fill" => Ok(Self::Stretch),
                _ => Err(wezterm_dynamic::Error::Message(format!(
                    "expected one of 'top', 'middle', 'bottom' or 'stretch', but got {}",
                    s
                ))),
            },
            other => Err(wezterm_dynamic::Error::Message(format!(
                "expected one of 'top', 'middle', 'bottom' or 'stretch', but got {}",
                other.variant_name()
            ))),
        }
    }
}

impl ToDynamic for ElementAlign {
    fn to_dynamic(&self) -> Value {
        Value::String(self.as_str().to_string())
    }
}

/// The quarter-ellipse drawn at the top corners of a tab.
#[derive(Debug, Clone, Copy, PartialEq, FromDynamic, ToDynamic)]
pub struct CornerRadius {
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_corner")]
    pub width: Dimension,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_corner")]
    pub height: Dimension,
}

const fn default_corner() -> Dimension {
    cells(0.5)
}

impl Default for CornerRadius {
    fn default() -> Self {
        Self {
            width: default_corner(),
            height: default_corner(),
        }
    }
}

/// Geometry of a tab.
#[derive(Debug, Clone, Copy, PartialEq, FromDynamic, ToDynamic)]
pub struct TabElementConfig {
    #[dynamic(default = "default_tab_padding")]
    pub padding: BoxDim,
    #[dynamic(default = "default_tab_margin")]
    pub margin: BoxDim,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_one_pixel")]
    pub border_width: Dimension,
    #[dynamic(default)]
    pub corner_radius: CornerRadius,
    /// How much shorter than an active tab an inactive tab is drawn.
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_inactive_inset")]
    pub inactive_bottom_inset: Dimension,
}

const fn default_one_pixel() -> Dimension {
    px(1.)
}
const fn default_inactive_inset() -> Dimension {
    cells(0.33)
}
const fn default_tab_padding() -> BoxDim {
    BoxDim::new(cells(0.5), cells(0.5), cells(0.2), cells(0.25))
}
const fn default_tab_margin() -> BoxDim {
    BoxDim::new(px(0.), px(0.), cells(0.2), px(0.))
}

impl Default for TabElementConfig {
    fn default() -> Self {
        Self {
            padding: default_tab_padding(),
            margin: default_tab_margin(),
            border_width: default_one_pixel(),
            corner_radius: CornerRadius::default(),
            inactive_bottom_inset: default_inactive_inset(),
        }
    }
}

/// Geometry of a button whose content is a vector glyph: the new-tab
/// `+` and the per-tab `x`. A percentage `glyph_size` is measured
/// against the cell height.
#[derive(Debug, Clone, Copy, PartialEq, FromDynamic, ToDynamic)]
pub struct GlyphButtonConfig {
    #[dynamic(default = "default_middle")]
    pub vertical_align: ElementAlign,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_half")]
    pub glyph_size: Dimension,
    #[dynamic(default = "BoxDim::zero")]
    pub padding: BoxDim,
    #[dynamic(default = "BoxDim::zero")]
    pub margin: BoxDim,
    #[dynamic(try_from = "crate::units::PixelUnit", default = "default_zero_dim")]
    pub border_width: Dimension,
    /// Stroke weight of the glyph. Unset uses the font's underline
    /// thickness, but never less than 2px, which reads as bold at
    /// small glyph sizes.
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)]
    pub line_width: Option<Dimension>,
}

const fn default_middle() -> ElementAlign {
    ElementAlign::Middle
}
const fn default_half() -> Dimension {
    Dimension::Percent(0.5)
}
const fn default_zero_dim() -> Dimension {
    px(0.)
}

impl GlyphButtonConfig {
    pub fn new_tab_default() -> Self {
        Self {
            vertical_align: ElementAlign::Middle,
            glyph_size: default_half(),
            padding: BoxDim::new(cells(0.5), cells(0.5), cells(0.2), cells(0.25)),
            margin: BoxDim::new(cells(0.5), px(0.), cells(0.2), px(0.)),
            border_width: default_one_pixel(),
            line_width: None,
        }
    }

    pub fn close_tab_default() -> Self {
        Self {
            vertical_align: ElementAlign::Middle,
            glyph_size: default_half(),
            padding: BoxDim::uniform(cells(0.25)),
            margin: BoxDim::new(cells(0.5), px(0.), px(0.), px(0.)),
            border_width: default_zero_dim(),
            line_width: None,
        }
    }
}

impl Default for GlyphButtonConfig {
    fn default() -> Self {
        Self::new_tab_default()
    }
}

/// Minimize / maximize / close. Sizes are optional because the
/// defaults differ per button style; leaving them unset keeps whatever
/// the selected style asks for.
#[derive(Debug, Clone, Copy, PartialEq, FromDynamic, ToDynamic)]
pub struct WindowButtonsConfig {
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)]
    pub glyph_size: Option<Dimension>,
    #[dynamic(default)]
    pub padding: Option<BoxDim>,
    /// Where the buttons sit when they are shorter than the bar.
    /// `Stretch` grows their vertical padding to fill it, so that the
    /// hover highlight and the click target cover the whole height.
    #[dynamic(default = "default_top")]
    pub vertical_align: ElementAlign,
    #[dynamic(try_from = "crate::units::OptPixelUnit", default)]
    pub line_width: Option<Dimension>,
}

const fn default_top() -> ElementAlign {
    ElementAlign::Top
}

impl Default for WindowButtonsConfig {
    fn default() -> Self {
        Self {
            glyph_size: None,
            padding: None,
            vertical_align: default_top(),
            line_width: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(value: Value) -> BoxDim {
        BoxDim::from_dynamic(&value, Default::default()).expect("from dynamic")
    }

    fn s(v: &str) -> Value {
        Value::String(v.to_string())
    }

    fn table(pairs: &[(&str, &str)]) -> Value {
        let mut obj: Object = BTreeMap::new().into();
        for (k, v) in pairs {
            obj.insert(s(k), s(v));
        }
        Value::Object(obj)
    }

    #[test]
    fn box_dim_scalar_applies_to_all_sides() {
        assert_eq!(parse(s("5px")), BoxDim::uniform(px(5.)));
        assert_eq!(parse(Value::I64(5)), BoxDim::uniform(px(5.)));
        assert_eq!(parse(s("0.25cell")), BoxDim::uniform(cells(0.25)));
    }

    #[test]
    fn box_dim_table_names_sides() {
        assert_eq!(
            parse(table(&[
                ("left", "12px"),
                ("right", "4px"),
                ("top", "3px"),
                ("bottom", "4px")
            ])),
            BoxDim::new(px(12.), px(4.), px(3.), px(4.))
        );
    }

    #[test]
    fn box_dim_table_omitted_sides_are_zero() {
        assert_eq!(
            parse(table(&[("left", "12px")])),
            BoxDim::new(px(12.), px(0.), px(0.), px(0.))
        );
    }

    #[test]
    fn box_dim_accepts_negative_values() {
        assert_eq!(
            parse(table(&[("bottom", "-2px")])),
            BoxDim::new(px(0.), px(0.), px(0.), px(-2.))
        );
    }

    #[test]
    fn box_dim_round_trips_through_to_dynamic() {
        let original = BoxDim::new(px(12.), px(4.), cells(0.2), px(-2.));
        assert_eq!(parse(original.to_dynamic()), original);
    }

    #[test]
    fn element_align_is_case_insensitive() {
        for (input, expect) in [
            ("Bottom", ElementAlign::Bottom),
            ("bottom", ElementAlign::Bottom),
            ("MIDDLE", ElementAlign::Middle),
            ("stretch", ElementAlign::Stretch),
        ] {
            assert_eq!(
                ElementAlign::from_dynamic(
                    &Value::String(input.to_string()),
                    Default::default()
                )
                .unwrap(),
                expect
            );
        }
    }

    #[test]
    fn defaults_match_the_values_upstream_hardcoded() {
        let tab = TabElementConfig::default();
        assert_eq!(
            tab.padding,
            BoxDim::new(cells(0.5), cells(0.5), cells(0.2), cells(0.25))
        );
        assert_eq!(tab.margin.top, cells(0.2));
        assert_eq!(tab.border_width, px(1.));
        assert_eq!(tab.inactive_bottom_inset, cells(0.33));

        let new_tab = GlyphButtonConfig::new_tab_default();
        assert_eq!(new_tab.glyph_size, Dimension::Percent(0.5));
        assert_eq!(new_tab.margin.left, cells(0.5));
        assert_eq!(new_tab.vertical_align, ElementAlign::Middle);

        let close = GlyphButtonConfig::close_tab_default();
        assert_eq!(close.padding, BoxDim::uniform(cells(0.25)));
        assert_eq!(close.margin.left, cells(0.5));
    }
}
