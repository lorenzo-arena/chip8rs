/* TODO : move to extern crate? */
/* TODO : make algorithm templated to work with u8 representation of a rgb */
#[derive(Debug, Copy, Clone)]
pub struct RGBPixel {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct HSLPixel {
    pub h: i32,
    pub s: f32,
    pub l: f32,
}

fn get_min(a: f32, b: f32) -> f32 {
    if a <= b {
        a
    } else {
        b
    }
}

fn get_max(a: f32, b: f32) -> f32 {
    if a >= b {
        a
    } else {
        b
    }
}

pub fn rgb_to_hsl(rgb: &RGBPixel) -> HSLPixel {
    let mut hsl = HSLPixel {
        h: 0,
        s: 0.0,
        l: 0.0,
    };

    // 1. Calculate the min and max and mean to get the luminance
    let min = get_min(get_min(rgb.r, rgb.g), rgb.b);
    let max = get_max(get_max(rgb.r, rgb.g), rgb.b);

    let chroma = max - min;

    hsl.l = (min + max) / 2.0;

    // 2. If min and max are equal, we have a shade of gray and hue and saturation are 0;
    // otherwise they must be calculated
    if min == max {
        hsl.s = 0.0;
        hsl.h = 0;
    } else {
        if max == rgb.r {
            hsl.h = (60.0 * ((rgb.g - rgb.b) / chroma)).round() as i32;
        } else if max == rgb.g {
            hsl.h = (60.0 * (2.0 + ((rgb.b - rgb.r) / chroma))).round() as i32;
        } else {
            hsl.h = (60.0 * (4.0 + ((rgb.r - rgb.g) / chroma))).round() as i32;
        }

        if hsl.h < 0 {
            hsl.h += 360;
        } else if hsl.h > 360 {
            hsl.h -= 360;
        }

        hsl.s = chroma / (1.0 - ((2.0 * max) - chroma - 1.0).abs());
    }

    hsl
}

pub fn hsl_to_rgb(hsl: &HSLPixel) -> RGBPixel {
    let mut rgb = RGBPixel {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };

    // If saturation is zero we have a shade of gray
    if hsl.s == 0.0 {
        rgb.r = hsl.l;
        rgb.g = hsl.l;
        rgb.b = hsl.l;
    } else {
        let chroma = (1.0 - ((2.0 * hsl.l) - 1.0).abs()) * hsl.s;

        let hue = hsl.h % 360;
        let hue_f = (hue as f32) / 60.0;

        let x = chroma * (1.0 - ((hue_f % 2.0) - 1.0).abs());

        let mut red = 0.0;
        let mut green = 0.0;
        let mut blue = 0.0;

        if (0.0 <= hue_f) && (hue_f <= 1.0) {
            red = chroma;
            green = x;
            blue = 0.0;
        } else if (1.0 <= hue_f) && (hue_f <= 2.0) {
            red = x;
            green = chroma;
            blue = 0.0;
        } else if (2.0 <= hue_f) && (hue_f <= 3.0) {
            red = 0.0;
            green = chroma;
            blue = x;
        } else if (3.0 <= hue_f) && (hue_f <= 4.0) {
            red = 0.0;
            green = x;
            blue = chroma;
        } else if (4.0 <= hue_f) && (hue_f <= 5.0) {
            red = x;
            green = 0.0;
            blue = chroma;
        } else if (5.0 <= hue_f) && (hue_f <= 6.0) {
            red = chroma;
            green = 0.0;
            blue = x;
        }

        let m = hsl.l - (chroma / 2.0);

        rgb.r = red + m;
        rgb.g = green + m;
        rgb.b = blue + m;
    }

    rgb
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    const PIXELS: [(RGBPixel, HSLPixel); 5] = [
        (
            RGBPixel {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            },
            HSLPixel {
                h: 0,
                s: 1.0,
                l: 0.5,
            },
        ),
        (
            RGBPixel {
                r: 0.0,
                g: 1.0,
                b: 0.0,
            },
            HSLPixel {
                h: 120,
                s: 1.0,
                l: 0.5,
            },
        ),
        (
            RGBPixel {
                r: 0.0,
                g: 0.0,
                b: 1.0,
            },
            HSLPixel {
                h: 240,
                s: 1.0,
                l: 0.5,
            },
        ),
        (
            RGBPixel {
                r: 0.0,
                g: 1.0,
                b: 1.0,
            },
            HSLPixel {
                h: 180,
                s: 1.0,
                l: 0.5,
            },
        ),
        (
            RGBPixel {
                r: 0.25,
                g: 0.875,
                b: 0.8125,
            },
            HSLPixel {
                h: 174,
                s: 0.71428573,
                l: 0.5625,
            },
        ),
    ];

    #[test]
    fn rgb_to_hsl_conversion() {
        for set in PIXELS {
            let (rgb, hsl) = set;

            let result = rgb_to_hsl(&rgb);
            assert_eq!(result.h, hsl.h);
            assert_approx_eq!(result.s, hsl.s);
            assert_approx_eq!(result.l, hsl.l);
        }
    }

    #[test]
    fn hsl_to_rgb_conversion() {
        for set in PIXELS {
            let (rgb, hsl) = set;

            let result = hsl_to_rgb(&hsl);
            assert_approx_eq!(result.r, rgb.r);
            assert_approx_eq!(result.g, rgb.g);
            assert_approx_eq!(result.b, rgb.b);
        }
    }
}
