//! Rust Clock View: time calculations, Canvas drawing and particle effects.
//! The small JavaScript entrypoint only handles browser lifecycle events.

use std::f64::consts::{FRAC_PI_2, TAU};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement};

const BACKGROUND: &str = "#0b0f18";
const PARTICLE_COUNT: usize = 60;

#[derive(Debug, Clone, Copy)]
struct ClockTime {
    hours: u32,
    minutes: u32,
    seconds: u32,
    smooth_seconds: f64,
    smooth_minutes: f64,
    smooth_hours: f64,
}

impl ClockTime {
    fn new(hours: u32, minutes: u32, seconds: u32, milliseconds: u32) -> Self {
        let smooth_seconds = seconds as f64 + milliseconds as f64 / 1000.0;
        let smooth_minutes = minutes as f64 + smooth_seconds / 60.0;
        let smooth_hours = (hours % 12) as f64 + smooth_minutes / 60.0;
        Self {
            hours,
            minutes,
            seconds,
            smooth_seconds,
            smooth_minutes,
            smooth_hours,
        }
    }
}

fn format_time(hours: u32, minutes: u32) -> String {
    format!("{hours:02}:{minutes:02}")
}

fn format_date(year: u32, month: u32, day: u32, weekday: u32) -> String {
    const WEEKDAYS: [&str; 7] = [
        "日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日",
    ];
    let weekday = WEEKDAYS.get(weekday as usize).copied().unwrap_or("");
    format!("{year}年{month}月{day}日 {weekday}")
}

fn time_hue(hours: u32, minutes: u32) -> f64 {
    ((hours * 60 + minutes) as f64 / 1440.0) * 360.0
}

fn should_explode(previous: Option<i64>, current: i64) -> bool {
    previous.is_some_and(|minute| minute != current)
}

#[derive(Clone, Copy, Debug)]
struct Geometry {
    x: f64,
    y: f64,
    radius: f64,
}

fn clock_geometry(width: f64, height: f64) -> Geometry {
    Geometry {
        x: width / 2.0,
        y: height / 2.0,
        radius: (width * 0.42).min(height * 0.34).min(214.0),
    }
}

#[derive(Debug, Clone, Copy)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    hue: f64,
    age: f64,
    life: f64,
}

fn advance_particle(particle: &mut Particle, elapsed_ms: f64) {
    let dt = elapsed_ms / 1000.0;
    particle.x += particle.vx * dt;
    particle.y += particle.vy * dt;
    particle.age += elapsed_ms;
}

fn create_particles(geometry: Geometry, hue: f64) -> Vec<Particle> {
    (0..PARTICLE_COUNT)
        .map(|i| {
            let angle = TAU * i as f64 / PARTICLE_COUNT as f64
                - FRAC_PI_2
                + (js_sys::Math::random() - 0.5) * 0.28;
            let speed = 55.0 + js_sys::Math::random() * 115.0;
            Particle {
                x: geometry.x + angle.cos() * geometry.radius * 1.11,
                y: geometry.y + angle.sin() * geometry.radius * 1.11,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                hue,
                age: 0.0,
                life: 650.0 + js_sys::Math::random() * 550.0,
            }
        })
        .collect()
}

fn draw_ring(
    ctx: &CanvasRenderingContext2d,
    x: f64,
    y: f64,
    radius: f64,
    ratio: f64,
    color: &str,
    width: f64,
) {
    if ratio <= 0.0 {
        return;
    }
    ctx.begin_path();
    let _ = ctx.arc(x, y, radius, -FRAC_PI_2, TAU * ratio - FRAC_PI_2);
    ctx.set_stroke_style_str(color);
    ctx.set_line_width(width);
    ctx.set_line_cap("round");
    ctx.stroke();
}

fn draw_face(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    geometry: Geometry,
    time: ClockTime,
    reduced_motion: bool,
) {
    ctx.set_fill_style_str(if reduced_motion {
        BACKGROUND
    } else {
        "rgba(11, 15, 24, 0.28)"
    });
    ctx.fill_rect(0.0, 0.0, width, height);

    let hour_radius = geometry.radius * 0.65;
    let minute_radius = geometry.radius * 0.83;
    let second_radius = geometry.radius;
    let track_width = (geometry.radius * 0.016).max(2.0);

    for radius in [hour_radius, minute_radius, second_radius] {
        draw_ring(
            ctx,
            geometry.x,
            geometry.y,
            radius,
            1.0,
            "rgba(235, 240, 255, 0.085)",
            track_width,
        );
    }

    let hue = time_hue(time.hours, time.minutes);
    draw_ring(
        ctx,
        geometry.x,
        geometry.y,
        hour_radius,
        time.smooth_hours / 12.0,
        &format!("hsl({hue}, 86%, 68%)"),
        (geometry.radius * 0.040).max(5.0),
    );
    draw_ring(
        ctx,
        geometry.x,
        geometry.y,
        minute_radius,
        time.smooth_minutes / 60.0,
        "rgba(229, 237, 250, 0.88)",
        (geometry.radius * 0.027).max(4.0),
    );
    draw_ring(
        ctx,
        geometry.x,
        geometry.y,
        second_radius,
        if reduced_motion {
            time.seconds as f64 / 60.0
        } else {
            time.smooth_seconds / 60.0
        },
        "rgba(158, 188, 220, 0.78)",
        (geometry.radius * 0.014).max(2.0),
    );

    ctx.set_fill_style_str("rgba(219, 232, 250, 0.65)");
    for i in 0..time.seconds {
        let angle = TAU * i as f64 / 60.0 - FRAC_PI_2;
        let x = geometry.x + angle.cos() * geometry.radius * 1.11;
        let y = geometry.y + angle.sin() * geometry.radius * 1.11;
        ctx.begin_path();
        let _ = ctx.arc(x, y, (geometry.radius * 0.009).max(1.0), 0.0, TAU);
        ctx.fill();
    }
}

fn draw_particle(ctx: &CanvasRenderingContext2d, particle: &Particle, radius: f64) {
    let opacity = (1.0 - particle.age / particle.life).max(0.0);
    ctx.set_fill_style_str(&format!(
        "hsla({}, 88%, 72%, {})",
        particle.hue, opacity
    ));
    ctx.begin_path();
    let _ = ctx.arc(particle.x, particle.y, (radius * 0.012).max(1.5), 0.0, TAU);
    ctx.fill();
}

/// Exposed to the browser by wasm-bindgen; all clock rendering happens in Rust.
#[wasm_bindgen]
pub struct ClockApp {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    digital: Element,
    date: Element,
    width: f64,
    height: f64,
    last_minute: Option<i64>,
    last_frame: Option<f64>,
    particles: Vec<Particle>,
}

#[wasm_bindgen]
impl ClockApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ClockApp, JsValue> {
        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is unavailable"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is unavailable"))?;
        let canvas: HtmlCanvasElement = document
            .get_element_by_id("clock-canvas")
            .ok_or_else(|| JsValue::from_str("clock-canvas was not found"))?
            .dyn_into()?;
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Canvas 2D is unavailable"))?
            .dyn_into()?;
        let digital = document
            .get_element_by_id("digital-time")
            .ok_or_else(|| JsValue::from_str("digital-time was not found"))?;
        let date = document
            .get_element_by_id("date-text")
            .ok_or_else(|| JsValue::from_str("date-text was not found"))?;

        let mut app = ClockApp {
            canvas,
            ctx,
            digital,
            date,
            width: 1.0,
            height: 1.0,
            last_minute: None,
            last_frame: None,
            particles: Vec::new(),
        };
        app.resize()?;
        Ok(app)
    }

    /// Update backing pixels for HiDPI devices without changing CSS coordinates.
    pub fn resize(&mut self) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        self.width = window.inner_width()?.as_f64().unwrap_or(1.0).max(1.0);
        self.height = window.inner_height()?.as_f64().unwrap_or(1.0).max(1.0);
        let dpr = window.device_pixel_ratio().clamp(1.0, 2.0);
        self.canvas.set_width((self.width * dpr).round() as u32);
        self.canvas.set_height((self.height * dpr).round() as u32);
        self.ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0)?;
        self.ctx.set_fill_style_str(BACKGROUND);
        self.ctx.fill_rect(0.0, 0.0, self.width, self.height);
        self.particles.clear();
        Ok(())
    }

    /// Called by requestAnimationFrame; the browser timestamp controls particle movement.
    pub fn frame(&mut self, timestamp: f64, reduced_motion: bool) {
        let date = js_sys::Date::new_0();
        let time = ClockTime::new(
            date.get_hours(),
            date.get_minutes(),
            date.get_seconds(),
            date.get_milliseconds(),
        );
        let minute_key = (date.get_time() / 60_000.0).floor() as i64;

        let time_label = format_time(time.hours, time.minutes);
        if self.digital.text_content().as_deref() != Some(&time_label) {
            self.digital.set_text_content(Some(&time_label));
        }
        let date_label = format_date(
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date(),
            date.get_day(),
        );
        if self.date.text_content().as_deref() != Some(&date_label) {
            self.date.set_text_content(Some(&date_label));
        }

        let elapsed_ms = self
            .last_frame
            .map(|previous| (timestamp - previous).clamp(0.0, 50.0))
            .unwrap_or(16.0);
        self.last_frame = Some(timestamp);

        let geometry = clock_geometry(self.width, self.height);
        draw_face(
            &self.ctx,
            self.width,
            self.height,
            geometry,
            time,
            reduced_motion,
        );

        if reduced_motion {
            self.particles.clear();
        } else {
            if should_explode(self.last_minute, minute_key) {
                self.particles.extend(create_particles(geometry, time_hue(time.hours, time.minutes)));
            }
            for particle in &mut self.particles {
                advance_particle(particle, elapsed_ms);
            }
            self.particles.retain(|particle| particle.age < particle.life);
            for particle in &self.particles {
                draw_particle(&self.ctx, particle, geometry.radius);
            }
        }
        self.last_minute = Some(minute_key);
    }

    /// Prevent a large delta after the tab resumes.
    pub fn pause(&mut self) {
        self.last_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_components_have_fractional_progress() {
        let time = ClockTime::new(13, 30, 15, 500);
        assert_eq!(time.hours, 13);
        assert_eq!(time.minutes, 30);
        assert_eq!(time.seconds, 15);
        assert_eq!(time.smooth_seconds, 15.5);
        assert!(time.smooth_minutes > 30.0);
        assert!(time.smooth_hours > 1.0);
        assert_eq!(format_time(time.hours, time.minutes), "13:30");
    }

    #[test]
    fn minute_boundary_triggers_only_after_first_frame() {
        assert!(!should_explode(None, 5));
        assert!(!should_explode(Some(5), 5));
        assert!(should_explode(Some(5), 6));
        assert!(!should_explode(Some(6), 6));
    }

    #[test]
    fn hue_tracks_time_of_day() {
        assert_eq!(time_hue(0, 0), 0.0);
        assert_eq!(time_hue(12, 0), 180.0);
        assert!(time_hue(23, 59) < 360.0);
    }

    #[test]
    fn clock_geometry_fits_mobile_and_desktop() {
        let mobile = clock_geometry(320.0, 640.0);
        let desktop = clock_geometry(1920.0, 1080.0);
        assert!(mobile.radius * 1.15 < 160.0);
        assert!(desktop.radius <= 214.0);
        assert_eq!(mobile.x, 160.0);
        assert_eq!(desktop.y, 540.0);
    }

    #[test]
    fn particle_movement_depends_on_elapsed_time() {
        let mut particle = Particle {
            x: 50.0,
            y: 100.0,
            vx: 10.0,
            vy: -20.0,
            hue: 180.0,
            age: 0.0,
            life: 1000.0,
        };
        advance_particle(&mut particle, 250.0);
        assert_eq!(particle.x, 52.5);
        assert_eq!(particle.y, 95.0);
        assert_eq!(particle.age, 250.0);
    }

    #[test]
    fn japanese_date_format_is_stable() {
        assert_eq!(format_date(2026, 11, 17, 2), "2026年11月17日 火曜日");
    }
}
