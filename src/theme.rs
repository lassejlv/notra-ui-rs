//! Notra's design tokens, extracted from the live ui.usenotra.com stylesheet.
//!
//! Every colour is sRGB with straight alpha. OKLCH values from the CSS were
//! converted offline, so the numbers here are exactly what a browser paints.

/// An sRGB colour with straight alpha, each channel in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0x0000_0000);
    pub const WHITE: Self = Self::rgb(0xffffff);
    pub const BLACK: Self = Self::rgb(0x000000);

    /// `0xRRGGBB`, opaque.
    pub const fn rgb(hex: u32) -> Self {
        Self::rgba((hex << 8) | 0xff)
    }

    /// `0xRRGGBBAA`.
    pub const fn rgba(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xff) as f32 / 255.0,
            g: ((hex >> 16) & 0xff) as f32 / 255.0,
            b: ((hex >> 8) & 0xff) as f32 / 255.0,
            a: (hex & 0xff) as f32 / 255.0,
        }
    }

    /// The same colour at `alpha` (CSS `color/alpha`, Tailwind `text-foreground/60`).
    pub const fn alpha(self, alpha: f32) -> Self {
        Self { a: alpha, ..self }
    }

    /// Multiplies the existing alpha.
    pub fn fade(self, factor: f32) -> Self {
        Self {
            a: self.a * factor,
            ..self
        }
    }

    /// CSS `color-mix(in oklab, self, other p%)`: `t` is the share of `other`.
    /// Alpha mixes premultiplied, as browsers do.
    pub fn mix(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let (a, b) = (self.to_oklab(), other.to_oklab());
        let alpha = self.a * (1.0 - t) + other.a * t;
        if alpha <= 0.0 {
            return Self::TRANSPARENT;
        }
        let lerp = |x: f32, y: f32| (x * self.a * (1.0 - t) + y * other.a * t) / alpha;
        Self::from_oklab(
            [lerp(a[0], b[0]), lerp(a[1], b[1]), lerp(a[2], b[2])],
            alpha,
        )
    }

    /// CSS `filter: brightness(f)` applied to this colour.
    pub fn brightness(self, factor: f32) -> Self {
        Self {
            r: (self.r * factor).clamp(0.0, 1.0),
            g: (self.g * factor).clamp(0.0, 1.0),
            b: (self.b * factor).clamp(0.0, 1.0),
            a: self.a,
        }
    }

    /// Paints `self` over `base` (source-over) and returns an opaque-as-base result.
    pub fn over(self, base: Self) -> Self {
        let a = self.a + base.a * (1.0 - self.a);
        if a <= 0.0 {
            return Self::TRANSPARENT;
        }
        let ch = |s: f32, d: f32| (s * self.a + d * base.a * (1.0 - self.a)) / a;
        Self {
            r: ch(self.r, base.r),
            g: ch(self.g, base.g),
            b: ch(self.b, base.b),
            a,
        }
    }

    /// Linear-light channels, for renderers that take linear sRGB.
    pub fn to_linear(self) -> [f32; 4] {
        [
            to_linear(self.r),
            to_linear(self.g),
            to_linear(self.b),
            self.a,
        ]
    }

    /// Converts an OKLCH colour (`l` in `0..=1`, `c` chroma, `h` degrees).
    pub fn oklch(l: f32, c: f32, h: f32) -> Self {
        let (s, co) = h.to_radians().sin_cos();
        Self::from_oklab([l, c * co, c * s], 1.0)
    }

    fn to_oklab(self) -> [f32; 3] {
        let [r, g, b, _] = self.to_linear();
        let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
        let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
        let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
        [
            0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        ]
    }

    fn from_oklab([l, a, b]: [f32; 3], alpha: f32) -> Self {
        let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
        let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
        let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;
        let (l, m, s) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
        let r = 4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s;
        let g = -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s;
        let b = -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s;
        Self {
            r: from_linear(r),
            g: from_linear(g),
            b: from_linear(b),
            a: alpha,
        }
    }
}

fn to_linear(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn from_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// One CSS `box-shadow` layer, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

impl Shadow {
    pub const fn drop(y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            x: 0.0,
            y,
            blur,
            spread,
            color,
            inset: false,
        }
    }

    pub const fn inset(y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            x: 0.0,
            y,
            blur,
            spread,
            color,
            inset: true,
        }
    }
}

/// Light or dark appearance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    #[default]
    Light,
    Dark,
}

/// Syntax colours used by the code block (sugar-high's `--sh-*` variables).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Syntax {
    pub class: Color,
    pub identifier: Color,
    pub sign: Color,
    pub property: Color,
    pub entity: Color,
    pub jsx_literals: Color,
    pub string: Color,
    pub keyword: Color,
    pub comment: Color,
}

/// The full Notra palette for one appearance.
///
/// Names follow the CSS variables (`--muted-foreground` is `muted_foreground`), so
/// a Tailwind class such as `text-muted-foreground` maps to one field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub appearance: Appearance,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub border: Color,
    /// The page the components sit on (the preview canvas of ui.usenotra.com).
    pub canvas: Color,
    /// shadcn's `accent`: the hover fill of menu rows and ghost controls.
    pub accent: Color,
    pub accent_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub ring: Color,
    pub destructive: Color,
    pub destructive_foreground: Color,
    pub popover: Color,
    pub popover_foreground: Color,
    pub secondary: Color,
    pub secondary_foreground: Color,
    pub input: Color,
    pub code: Color,
    pub code_foreground: Color,
    pub sidebar: Color,
    pub sidebar_foreground: Color,
    pub sidebar_accent: Color,
    pub sidebar_accent_foreground: Color,
    pub sidebar_border: Color,
    pub sidebar_ring: Color,
    /// `--shell`, the grey frame of every dualtone surface.
    pub shell: Color,
    pub shell_border: Color,
    pub table_card: Color,
    pub table_card_border: Color,
    pub table_row_border: Color,
    pub card: Color,
    pub tile: Color,
    pub tile_border: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub brand_violet: Color,
    pub brand_lavender: Color,
    pub brand_ink: Color,
    pub brand_cream: Color,
    pub syntax: Syntax,
    /// `--table-card-shadow`: the lift of a dualtone body card.
    pub table_card_shadow: &'static [Shadow],
    /// `--surface-lift`.
    pub surface_lift: &'static [Shadow],
}

const LIGHT_CARD_SHADOW: &[Shadow] = &[
    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x17171c0f)),
    Shadow::drop(2.0, 8.0, -2.0, Color::rgba(0x17171c0f)),
];
const DARK_CARD_SHADOW: &[Shadow] = &[
    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0d)),
    Shadow::inset(0.0, 0.0, 1.0, Color::rgba(0xffffff04)),
    Shadow::inset(12.0, 18.0, -16.0, Color::rgba(0xffffff09)),
];
const DARK_SURFACE_LIFT: &[Shadow] = &[
    Shadow::inset(1.0, 0.0, 0.0, Color::rgba(0xffffff0a)),
    Shadow::drop(1.0, 2.0, 0.0, Color::rgba(0x00000066)),
];

impl Theme {
    pub fn new(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self::light(),
            Appearance::Dark => Self::dark(),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.appearance == Appearance::Dark
    }

    pub const fn light() -> Self {
        Self {
            appearance: Appearance::Light,
            background: Color::rgb(0xffffff),
            foreground: Color::rgb(0x171717),
            muted: Color::rgb(0xf5f5f5),
            muted_foreground: Color::rgb(0x737373),
            border: Color::rgb(0xe5e5e5),
            canvas: Color::rgb(0xfafafa),
            accent: Color::rgb(0xf5f5f5),
            accent_foreground: Color::rgb(0x171717),
            primary: Color::rgb(0x7e4ce7),
            primary_foreground: Color::rgb(0xfefefe),
            ring: Color::rgb(0x7e4ce7),
            destructive: Color::rgb(0xe7000b),
            destructive_foreground: Color::rgb(0xfafafa),
            popover: Color::rgb(0xffffff),
            popover_foreground: Color::rgb(0x0a0a0a),
            secondary: Color::rgb(0xf5f5f5),
            secondary_foreground: Color::rgb(0x171717),
            input: Color::rgb(0xe5e5e5),
            code: Color::rgb(0xfafafa),
            code_foreground: Color::rgb(0x09090b),
            sidebar: Color::rgb(0xfafafa),
            sidebar_foreground: Color::rgb(0x0a0a0a),
            sidebar_accent: Color::rgb(0xf5f5f5),
            sidebar_accent_foreground: Color::rgb(0x171717),
            sidebar_border: Color::rgb(0xe5e5e5),
            sidebar_ring: Color::rgb(0x6d28d9),
            shell: Color::rgb(0xf3f3f2),
            shell_border: Color::rgb(0xe7e7e5),
            table_card: Color::rgb(0xffffff),
            table_card_border: Color::rgb(0xe5e5e5),
            table_row_border: Color::rgb(0xefefee),
            card: Color::rgb(0xffffff),
            tile: Color::rgb(0xfafafa),
            tile_border: Color::rgb(0xeaeaec),
            info: Color::rgb(0x2a75ba),
            success: Color::rgb(0x328455),
            warning: Color::rgb(0xa9632e),
            brand_violet: Color::rgb(0x8b5cf6),
            brand_lavender: Color::rgb(0xc8b2ee),
            brand_ink: Color::rgb(0x1e1e1e),
            brand_cream: Color::rgb(0xf6f3f1),
            syntax: Syntax {
                class: Color::rgb(0x4c6ebd),
                identifier: Color::rgb(0x484d58),
                sign: Color::rgb(0x83868c),
                property: Color::rgb(0xb94642),
                entity: Color::rgb(0x4c6ebd),
                jsx_literals: Color::rgb(0x794db6),
                string: Color::rgb(0x187c49),
                keyword: Color::rgb(0xa041b5),
                comment: Color::rgb(0x898f9c),
            },
            table_card_shadow: LIGHT_CARD_SHADOW,
            surface_lift: LIGHT_CARD_SHADOW,
        }
    }

    pub const fn dark() -> Self {
        Self {
            appearance: Appearance::Dark,
            background: Color::rgb(0x131316),
            foreground: Color::rgb(0xfafafa),
            muted: Color::rgb(0x252629),
            muted_foreground: Color::rgb(0xb3b4b8),
            border: Color::rgb(0x2f3033),
            canvas: Color::rgb(0x18181b),
            accent: Color::rgb(0x252629),
            accent_foreground: Color::rgb(0xfafafa),
            primary: Color::rgb(0x8b5cf6),
            primary_foreground: Color::rgb(0xfafafa),
            ring: Color::rgb(0x8b5cf6),
            destructive: Color::rgb(0xf54748),
            destructive_foreground: Color::rgb(0xfcfcfc),
            popover: Color::rgb(0x141518),
            popover_foreground: Color::rgb(0xfafafa),
            secondary: Color::rgb(0x252629),
            secondary_foreground: Color::rgb(0xfafafa),
            input: Color::rgb(0x343539),
            code: Color::rgb(0x18181b),
            code_foreground: Color::rgb(0xfafafa),
            sidebar: Color::rgb(0x0e0f12),
            sidebar_foreground: Color::rgb(0xfafafa),
            sidebar_accent: Color::rgb(0x1f2023),
            sidebar_accent_foreground: Color::rgb(0xa78bfa),
            sidebar_border: Color::rgb(0x26282b),
            sidebar_ring: Color::rgb(0xa78bfa),
            shell: Color::rgb(0x161616),
            shell_border: Color::rgb(0x232323),
            table_card: Color::rgb(0x1f1f1f),
            table_card_border: Color::rgb(0x333333),
            table_row_border: Color::rgb(0x292929),
            card: Color::rgb(0x18181b),
            tile: Color::rgb(0x18181b),
            tile_border: Color::rgb(0x222225),
            info: Color::rgb(0x61a3e6),
            success: Color::rgb(0x55b57c),
            warning: Color::rgb(0xd9884d),
            brand_violet: Color::rgb(0x8b5cf6),
            brand_lavender: Color::rgb(0xc8b2ee),
            brand_ink: Color::rgb(0x1e1e1e),
            brand_cream: Color::rgb(0xf6f3f1),
            syntax: Syntax {
                class: Color::rgb(0x87bafd),
                identifier: Color::rgb(0xced1d8),
                sign: Color::rgb(0x8f9299),
                property: Color::rgb(0xf49383),
                entity: Color::rgb(0x87bafd),
                jsx_literals: Color::rgb(0xc4a4fe),
                string: Color::rgb(0x79d28d),
                keyword: Color::rgb(0xe68cde),
                comment: Color::rgb(0x7a808d),
            },
            table_card_shadow: DARK_CARD_SHADOW,
            surface_lift: DARK_SURFACE_LIFT,
        }
    }
}

/// Corner radii (Tailwind's scale on ui.usenotra.com), in logical pixels.
pub mod radius {
    pub const XS: f32 = 2.0;
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 6.0;
    pub const LG: f32 = 8.0;
    pub const XL: f32 = 12.0;
    pub const XL2: f32 = 16.0;
    pub const XL3: f32 = 24.0;
    pub const XL4: f32 = 32.0;
    pub const FULL: f32 = 9999.0;
}

/// Tailwind's spacing unit: `p-4` is `4.0 * SPACING`.
pub const SPACING: f32 = 4.0;

/// One rem in logical pixels.
pub const REM: f32 = 16.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_channels_decode_in_order() {
        let c = Color::rgba(0x11223344);
        assert_eq!(
            (c.r, c.g, c.b, c.a),
            (17.0 / 255.0, 34.0 / 255.0, 51.0 / 255.0, 68.0 / 255.0)
        );
    }

    #[test]
    fn oklch_matches_the_converted_primary() {
        let c = Color::oklch(0.56, 0.22, 292.717);
        let p = Theme::light().primary;
        assert!((c.r - p.r).abs() < 0.01 && (c.g - p.g).abs() < 0.01 && (c.b - p.b).abs() < 0.01);
    }

    #[test]
    fn mix_endpoints_are_the_inputs() {
        let a = Color::rgb(0x7e4ce7);
        let m = a.mix(Color::BLACK, 0.0);
        assert!((m.r - a.r).abs() < 1e-3 && (m.b - a.b).abs() < 1e-3);
        let m = a.mix(Color::BLACK, 1.0);
        assert!(m.r < 1e-3 && m.g < 1e-3 && m.b < 1e-3);
    }
}
