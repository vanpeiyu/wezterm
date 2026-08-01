use crate::customglyph::*;
use crate::tabbar::{TabBarItem, TabEntry};
use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::*;

use crate::termwindow::render::window_buttons::window_button_element;
use crate::termwindow::{UIItem, UIItemType};
use crate::utilsprites::RenderMetrics;
use config::{
    Dimension, DimensionContext, ElementAlign, GlyphButtonConfig, TabBarColors, WindowFrameConfig,
};
use std::rc::Rc;
use wezterm_font::LoadedFont;
use wezterm_term::color::{ColorAttribute, ColorPalette};
use window::{IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle};

/// Bridge the config's four-sided dimension onto the box model's.
fn box_dim(b: &config::BoxDim) -> BoxDimension {
    BoxDimension {
        left: b.left,
        right: b.right,
        top: b.top,
        bottom: b.bottom,
    }
}

/// `Stretch` has no `VerticalAlign` of its own; it sits at the top and
/// is given a minimum height instead.
fn vertical_align(align: ElementAlign) -> VerticalAlign {
    match align {
        ElementAlign::Top | ElementAlign::Stretch => VerticalAlign::Top,
        ElementAlign::Middle => VerticalAlign::Middle,
        ElementAlign::Bottom => VerticalAlign::Bottom,
    }
}

fn stretch_to(align: ElementAlign, height: f32) -> Option<Dimension> {
    match align {
        ElementAlign::Stretch => Some(Dimension::Pixels(height)),
        _ => None,
    }
}

const X_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

const PLUS_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

impl crate::TermWindow {
    pub fn invalidate_fancy_tab_bar(&mut self) {
        self.fancy_tab_bar.take();
    }

    pub fn build_fancy_tab_bar(&self, palette: &ColorPalette) -> anyhow::Result<ComputedElement> {
        let tab_bar_height = self.tab_bar_pixel_height()?;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let frame: &WindowFrameConfig = &self.config.window_frame;

        // A percentage glyph size is measured against the cell height.
        let glyph_context = DimensionContext {
            dpi: self.dimensions.dpi as f32,
            pixel_max: metrics.cell_size.height as f32,
            pixel_cell: metrics.cell_size.height as f32,
        };
        let glyph_size = |cfg: &GlyphButtonConfig| -> Dimension {
            Dimension::Pixels(cfg.glyph_size.evaluate_as_pixels(glyph_context).max(1.))
        };
        let stroke = |cfg: &GlyphButtonConfig| -> isize {
            match cfg.line_width {
                Some(w) => (w.evaluate_as_pixels(glyph_context) as isize).max(1),
                None => metrics.underline_height.max(2),
            }
        };
        let new_tab_glyph = glyph_size(&frame.new_tab_button);
        let new_tab_stroke = stroke(&frame.new_tab_button);
        let colors = self
            .config
            .colors
            .as_ref()
            .and_then(|c| c.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default);

        let mut left_status = vec![];
        let mut left_eles = vec![];
        let mut right_eles = vec![];
        let bar_colors = ElementColors {
            border: BorderColor::default(),
            bg: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_bg
            } else {
                self.config.window_frame.inactive_titlebar_bg
            }
            .to_linear()
            .into(),
            text: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_fg
            } else {
                self.config.window_frame.inactive_titlebar_fg
            }
            .to_linear()
            .into(),
        };

        let item_to_elem = |item: &TabEntry| -> Element {
            let element = Element::with_line(&font, &item.title, palette);

            let bg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().background() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_bg(col)),
                });
            let fg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().foreground() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_fg(col)),
                });

            let new_tab = colors.new_tab();
            let new_tab_hover = colors.new_tab_hover();
            let active_tab = colors.active_tab();

            match item.item {
                TabBarItem::RightStatus | TabBarItem::LeftStatus | TabBarItem::None => element
                    .item_type(UIItemType::TabBar(TabBarItem::None))
                    .line_height(Some(frame.tab_bar_height_ratio))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.0),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.),
                        bottom: Dimension::Cells(0.),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .colors(bar_colors.clone()),
                TabBarItem::NewTabButton => Element::new(
                    &font,
                    ElementContent::Poly {
                        line_width: new_tab_stroke,
                        poly: SizedPoly {
                            poly: PLUS_BUTTON,
                            width: new_tab_glyph,
                            height: new_tab_glyph,
                        },
                    },
                )
                .vertical_align(vertical_align(frame.new_tab_button.vertical_align))
                .min_height(stretch_to(
                    frame.new_tab_button.vertical_align,
                    tab_bar_height,
                ))
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(box_dim(&frame.new_tab_button.margin))
                .padding(box_dim(&frame.new_tab_button.padding))
                .border(BoxDimension::new(frame.new_tab_button.border_width))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab.bg_color.to_linear().into(),
                    text: new_tab.fg_color.to_linear().into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab_hover.bg_color.to_linear().into(),
                    text: new_tab_hover.fg_color.to_linear().into(),
                })),
                TabBarItem::Tab { active, .. } if active => element
                    .vertical_align(vertical_align(frame.tab_vertical_align))
                    .min_height(stretch_to(frame.tab_vertical_align, tab_bar_height))
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(box_dim(&frame.tab.margin))
                    .padding(box_dim(&frame.tab.padding))
                    .border(BoxDimension::new(frame.tab.border_width))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: frame.tab.corner_radius.width,
                            height: frame.tab.corner_radius.height,
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: frame.tab.corner_radius.width,
                            height: frame.tab.corner_radius.height,
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly::none(),
                        bottom_right: SizedPoly::none(),
                    }))
                    .colors(ElementColors {
                        border: BorderColor::new(
                            bg_color
                                .unwrap_or_else(|| active_tab.bg_color.into())
                                .to_linear(),
                        ),
                        bg: bg_color
                            .unwrap_or_else(|| active_tab.bg_color.into())
                            .to_linear()
                            .into(),
                        text: fg_color
                            .unwrap_or_else(|| active_tab.fg_color.into())
                            .to_linear()
                            .into(),
                    }),
                TabBarItem::Tab { .. } => element
                    .vertical_align(vertical_align(frame.tab_vertical_align))
                    .min_height(stretch_to(frame.tab_vertical_align, tab_bar_height))
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(box_dim(&frame.tab.margin))
                    .padding(box_dim(&frame.tab.padding))
                    .border(BoxDimension::new(frame.tab.border_width))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: frame.tab.corner_radius.width,
                            height: frame.tab.corner_radius.height,
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: frame.tab.corner_radius.width,
                            height: frame.tab.corner_radius.height,
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        // An empty poly of this height is what makes an
                        // inactive tab sit shorter than an active one.
                        bottom_left: SizedPoly {
                            width: Dimension::Pixels(0.),
                            height: frame.tab.inactive_bottom_inset,
                            poly: &[],
                        },
                        bottom_right: SizedPoly {
                            width: Dimension::Pixels(0.),
                            height: frame.tab.inactive_bottom_inset,
                            poly: &[],
                        },
                    }))
                    .colors({
                        let inactive_tab = colors.inactive_tab();
                        let bg = bg_color
                            .unwrap_or_else(|| inactive_tab.bg_color.into())
                            .to_linear();
                        let edge = colors.inactive_tab_edge().to_linear();
                        ElementColors {
                            border: BorderColor {
                                left: bg,
                                right: edge,
                                top: bg,
                                bottom: bg,
                            },
                            bg: bg.into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        }
                    })
                    .hover_colors({
                        let inactive_tab_hover = colors.inactive_tab_hover();
                        Some(ElementColors {
                            border: BorderColor::new(
                                bg_color
                                    .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                    .to_linear(),
                            ),
                            bg: bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                    }),
                TabBarItem::WindowButton(button) => window_button_element(
                    button,
                    self.window_state.contains(window::WindowState::MAXIMIZED),
                    &font,
                    &metrics,
                    &self.config,
                    match frame.window_buttons.vertical_align {
                        ElementAlign::Stretch => Some(tab_bar_height),
                        _ => None,
                    },
                    self.dimensions.dpi as f32,
                ),
            }
        };

        let num_tabs: f32 = items
            .iter()
            .map(|item| match item.item {
                TabBarItem::NewTabButton | TabBarItem::Tab { .. } => 1.,
                _ => 0.,
            })
            .sum();
        let max_tab_width = ((self.dimensions.pixel_width as f32 / num_tabs)
            - (1.5 * metrics.cell_size.width as f32))
            .max(0.);

        // Reserve space for the native titlebar buttons
        if self
            .config
            .window_decorations
            .contains(::window::WindowDecorations::INTEGRATED_BUTTONS)
            && self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && !self.window_state.contains(window::WindowState::FULL_SCREEN)
        {
            left_status.push(
                Element::new(&font, ElementContent::Text("".to_string())).margin(BoxDimension {
                    left: Dimension::Cells(4.0), // FIXME: determine exact width of macos ... buttons
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                }),
            );
        }

        for item in items {
            match item.item {
                TabBarItem::LeftStatus => left_status.push(item_to_elem(item)),
                TabBarItem::None | TabBarItem::RightStatus => right_eles.push(item_to_elem(item)),
                TabBarItem::WindowButton(_) => {
                    if self.config.integrated_title_button_alignment
                        == IntegratedTitleButtonAlignment::Left
                    {
                        left_eles.push(item_to_elem(item))
                    } else {
                        right_eles.push(item_to_elem(item))
                    }
                }
                TabBarItem::Tab { tab_idx, active } => {
                    let mut elem = item_to_elem(item);
                    elem.max_width = Some(Dimension::Pixels(max_tab_width));
                    elem.content = match elem.content {
                        ElementContent::Text(_) => unreachable!(),
                        ElementContent::Poly { .. } => unreachable!(),
                        ElementContent::Children(mut kids) => {
                            if self.config.show_close_tab_button_in_tabs {
                                kids.push(make_x_button(
                                    &font,
                                    &colors,
                                    &frame.close_tab_button,
                                    glyph_size(&frame.close_tab_button),
                                    stroke(&frame.close_tab_button),
                                    tab_idx,
                                    active,
                                ));
                            }
                            ElementContent::Children(kids)
                        }
                    };
                    left_eles.push(elem);
                }
                _ => left_eles.push(item_to_elem(item)),
            }
        }

        let mut children = vec![];

        if !left_status.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(left_status))
                    .colors(bar_colors.clone()),
            );
        }

        let window_buttons_at_left = self
            .config
            .window_decorations
            .contains(window::WindowDecorations::INTEGRATED_BUTTONS)
            && (self.config.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Left
                || self.config.integrated_title_button_style
                    == IntegratedTitleButtonStyle::MacOsNative);

        // Unset reserves room for window buttons placed on the left.
        let left_padding = frame.tab_bar_left_padding.unwrap_or({
            if window_buttons_at_left {
                if self.config.integrated_title_button_style
                    == IntegratedTitleButtonStyle::MacOsNative
                {
                    if !self.window_state.contains(window::WindowState::FULL_SCREEN) {
                        Dimension::Pixels(70.0)
                    } else {
                        Dimension::Cells(0.5)
                    }
                } else {
                    Dimension::Pixels(0.0)
                }
            } else {
                Dimension::Cells(0.5)
            }
        });

        children.push(
            Element::new(&font, ElementContent::Children(left_eles))
                .vertical_align(VerticalAlign::Bottom)
                .colors(bar_colors.clone())
                .padding(BoxDimension {
                    left: left_padding,
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .zindex(1),
        );
        // Holds the window buttons and the right status.
        children.push(
            Element::new(&font, ElementContent::Children(right_eles))
                .vertical_align(vertical_align(frame.window_buttons.vertical_align))
                .min_height(stretch_to(
                    frame.window_buttons.vertical_align,
                    tab_bar_height,
                ))
                .colors(bar_colors.clone())
                .float(Float::Right),
        );

        let content = ElementContent::Children(children);

        let tabs = Element::new(&font, content)
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(self.dimensions.pixel_width as f32)))
            .min_height(Some(Dimension::Pixels(tab_bar_height)))
            .vertical_align(VerticalAlign::Bottom)
            .colors(bar_colors);

        let border = self.get_os_border();

        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    border.left.get() as f32,
                    0.,
                    self.dimensions.pixel_width as f32 - (border.left + border.right).get() as f32,
                    tab_bar_height,
                ),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &tabs,
        )?;

        computed.translate(euclid::vec2(
            0.,
            if self.config.tab_bar_at_bottom {
                self.dimensions.pixel_height as f32
                    - (computed.bounds.height() + border.bottom.get() as f32)
            } else {
                border.top.get() as f32
            },
        ));

        Ok(computed)
    }

    pub fn paint_fancy_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!("paint_fancy_tab_bar called but fancy_tab_bar is None")
        })?;
        let ui_items = computed.ui_items();

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;

        Ok(ui_items)
    }
}

fn make_x_button(
    font: &Rc<LoadedFont>,
    colors: &TabBarColors,
    cfg: &GlyphButtonConfig,
    glyph: Dimension,
    line_width: isize,
    tab_idx: usize,
    active: bool,
) -> Element {
    Element::new(
        &font,
        ElementContent::Poly {
            line_width,
            poly: SizedPoly {
                poly: X_BUTTON,
                width: glyph,
                height: glyph,
            },
        },
    )
    // Ensure that we draw our background over the
    // top of the rest of the tab contents
    .zindex(1)
    .vertical_align(vertical_align(cfg.vertical_align))
    .float(Float::Right)
    .item_type(UIItemType::CloseTab(tab_idx))
    .hover_colors({
        // Without an explicit color, take the one belonging to the
        // opposite tab state.
        let hover = colors.close_tab_button_hover().unwrap_or_else(|| {
            if active {
                colors.inactive_tab_hover()
            } else {
                colors.active_tab()
            }
        });

        Some(ElementColors {
            border: BorderColor::default(),
            bg: hover.bg_color.to_linear().into(),
            text: hover.fg_color.to_linear().into(),
        })
    })
    .padding(box_dim(&cfg.padding))
    .margin(box_dim(&cfg.margin))
    .border(BoxDimension::new(cfg.border_width))
}
