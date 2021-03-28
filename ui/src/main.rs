use druid::kurbo::{Size};
use druid::piet::{FontFamily, ImageFormat, InterpolationMode};
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};
use std::sync::Arc;
use std::ops::{Deref, Range};
use std::u64;

use gigatrace::trace::Ns;
use gigatrace::index::LongestEvent;
use gigatrace::{Trace, TrackInfo, self};

struct ViewMap {
    start: f64,
    scale: f64,
}

impl ViewMap {
    pub fn new(r: &Range<Ns>, width: f64) -> Self {
        Self { start: r.start as f64, scale: width/((r.end - r.start) as f64) }
    }

    pub fn to_x(&self, t: Ns) -> f64 {
        ((t as f64) - self.start) * self.scale
    }

    pub fn to_ns(&self, x: f64) -> f64 {
        self.start + (x / self.scale)
    }
}

struct ViewQuant {
    pub time_step: Ns,
}

impl ViewQuant {
    pub fn new(r: &Range<Ns>, width: f64) -> Self {
        let ns_per_px = (r.end-r.start)/(width as u64);
        let min_event_px = 2;
        let step = ns_per_px * min_event_px;
        let step = 1 << (64-step.leading_zeros());
        Self { time_step: u64::max(1, step) }
    }

    pub fn round_down(&self, x: Ns) -> Ns {
        x - (x % self.time_step)
    }

    pub fn quantize(&self, r: &Range<Ns>) -> Range<Ns> {
        self.round_down(r.start)..(self.round_down(r.end)+self.time_step)
    }
}

struct TimelineWidget {
    view_range: Range<Ns>,
}

impl TimelineWidget {
    fn paint_track(&self, ctx: &mut PaintCtx, trace: &Trace, env: &Env, track: &TrackInfo, size: Size) {
        // let rect = Rect::from_origin_size(Point::ORIGIN, size);
        // let fill_color = Color::rgb8(0x77, 0x00, 0x00);
        // ctx.fill(rect, &fill_color);

        let view = ViewMap::new(&self.view_range, size.width);
        let quant = ViewQuant::new(&self.view_range, size.width);
        let visible_events = gigatrace::aggregate_by_steps(&trace.pool, &track.track.block_locs, &track.zoom_index, quant.quantize(&self.view_range), quant.time_step);
        // for ev in track.track.events(&trace.pool) {
        for ev in visible_events.iter().filter_map(|x| x.0) {
            let ts = ev.ts.unpack();
            let dur = ev.dur.unpack();
            let (start, end) = if dur > quant.time_step {
                (ts, ts+dur)
            } else {
                let start = quant.round_down(ts);
                (start, start+quant.time_step)
            };
            // println!("{:?}: {} - {} -> {:.1} - {:.1} / {}", self.view_range, start, end, view.to_x(start), view.to_x(end), size.width);
            let rect = Rect::new(view.to_x(start),0.0,view.to_x(end),size.height);
            let fill_color = Color::rgb8(0x00, 0x00, (ev.kind % 250) as u8);
            ctx.fill(rect, &fill_color);
        }
    }

    fn zoom(&mut self, zoom_factor: f64, at_x: f64, size: Size) -> bool {
        let delta_time = self.view_range.end - self.view_range.start;
        let new_delta_time = (delta_time as f64) * zoom_factor; // TODO max zoom
        let view = ViewMap::new(&self.view_range, size.width);
        let zoom_time = view.to_ns(at_x);
        let r = at_x / size.width;
        let new_start = zoom_time - new_delta_time * r;
        let new_end = new_start + new_delta_time;
        self.view_range = (new_start as u64)..(new_end as u64);
        true
    }

    fn zoom_ratio(delta: f64) -> f64 {
        let wheel_zoom_speed = -0.02;
        let sign = if delta.is_sign_positive() { 1.0 } else { -1.0 };
        let delta_log = sign * (delta.abs()+1.0).log2();
        1.0-(delta_log*wheel_zoom_speed)
    }
}

impl Widget<Arc<Trace>> for TimelineWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut Arc<Trace>, _env: &Env) {
        if !ctx.is_handled() {
            if let Event::Wheel(mouse) = event {
                let factor = Self::zoom_ratio(mouse.wheel_delta.y);
                if self.zoom(factor, mouse.pos.x, ctx.size()) {
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &Arc<Trace>,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Arc<Trace>, _data: &Arc<Trace>, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Arc<Trace>,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Arc<Trace>, env: &Env) {
        // Clear background
        let size = ctx.size();
        let rect = Rect::from_origin_size(Point::ORIGIN, size);
        ctx.fill(rect, &Color::WHITE);

        let trace = data.deref();
        let track_height = 30.0;
        ctx.with_save(|ctx| {
            for track in &data.tracks {
                self.paint_track(ctx, trace, env, track, Size::new(size.width, track_height));
                ctx.transform(Affine::translate((0.0, track_height)));
            }
        });

        // Text is easy; in real use TextLayout should be stored in the widget
        // and reused.
        let text_color = Color::rgb8(0xFF, 0x00, 0x00);
        let mut layout = TextLayout::new("Gigatrace!");
        layout.set_font(FontDescriptor::new(FontFamily::SYSTEM_UI).with_size(12.0));
        layout.set_text_color(text_color);
        layout.rebuild_if_needed(ctx.text(), env);
        // layout.draw(ctx, (10.0, 10.0));
    }
}

pub fn main() {
    // let trace = Trace::demo_trace(5, 200_000_000);
    let trace = Trace::demo_trace(5, 2_000_000);
    let timeline = TimelineWidget {
        view_range: trace.time_bounds().unwrap_or(0..1000)
    };

    let window = WindowDesc::new(move || timeline).title(
        LocalizedString::new("gigatrace-window-title").with_placeholder("Gigatrace"),
    );
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(Arc::new(trace))
        .expect("launch failed");
}
