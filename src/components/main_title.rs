use rand::{thread_rng, Rng};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

const PIX2: f64 = std::f64::consts::PI * 2.0;
const CANVAS_WIDTH: u32 = 350;
const CANVAS_HEIGHT: u32 = 100;

struct Particle {
    x: f64,
    y: f64,
    velocity: f64,
    size: f32,
    speed: f64,
}

impl Particle {
    fn new() -> Self {
        let mut rng = thread_rng();
        let x = rng.gen_range(0f64..CANVAS_WIDTH as f64);
        let y = 0.0;
        let velocity = rng.gen_range(0.0..0.5);
        let size: f32 = rng.gen_range(0.0..1.0) + 1.0;
        Self {
            x,
            y,
            velocity,
            size,
            speed: 0.0,
        }
    }

    fn update(&mut self, map: &[Vec<f64>]) {
        let posx = self.x as usize;
        let posy = self.y as usize;
        self.speed = map[posy][posx];

        let movement = (2.5 - self.speed) + self.velocity;
        self.y += movement;
        if self.y >= CANVAS_HEIGHT as f64 {
            self.y = 0.0;
        }
    }

    fn render(&self, ctx: &CanvasRenderingContext2d) {
        ctx.begin_path();
        ctx.set_fill_style(&JsValue::from_str("white"));
        ctx.arc(self.x, self.y, self.size.into(), 0.0, PIX2)
            .unwrap();
        ctx.fill();
    }
}

fn relative_brightnes(r: f64, g: f64, b: f64) -> f64 {
    f64::sqrt((r * r) * 0.229 + (g * g) * 0.587 + (b * b) * 0.114) / 100.0
}

pub enum Msg {
    Init,
    Render,
}

#[derive(Properties, PartialEq, Clone)]
pub struct CanvasProps {
    pub title: String,
    pub width: usize,
    pub height: usize,
}

pub struct ParticleLogo {
    canvas: NodeRef,
    particles: Vec<Particle>,
    cb: Closure<dyn FnMut()>,
    brightnes_map: Vec<Vec<f64>>,
}

impl Component for ParticleLogo {
    type Message = Msg;
    type Properties = CanvasProps;
    fn create(ctx: &Context<Self>) -> Self {
        let particles = (0..1500).map(|_| Particle::new()).collect();
        let l = ctx.link().clone();
        let cb =
            Closure::wrap(Box::new(move || l.send_message(Msg::Render))
                as Box<dyn FnMut()>);

        ctx.link().send_message(Msg::Init);
        Self {
            particles,
            canvas: NodeRef::default(),
            cb,
            brightnes_map: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Init => {
                self.init(ctx);
                false
            }
            Msg::Render => {
                self.render();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <canvas id="canvas-title"
                width={ctx.props().width.to_string()}
                height={ctx.props().height.to_string()}
                ref={self.canvas.clone()}>
                </canvas>
        }
    }
}

impl ParticleLogo {
    fn init(&mut self, yew_ctx: &Context<Self>) {
        let title = &yew_ctx.props().title;
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let h = canvas.height() as usize;
        let w = canvas.width() as usize;
        let cctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        cctx.set_fill_style(&JsValue::from("white"));
        cctx.set_font("bold 60px Arial");
        let _ = cctx.fill_text(
            title,
            w as f64 - cctx.measure_text(title).unwrap().width(),
            h as f64 / 2.0 + 15.0,
        );

        let title_image_data =
            cctx.get_image_data(0.0, 0.0, w as f64, h as f64).unwrap();
        let buffer = title_image_data.data();
        let mut brightnes_map = Vec::new();
        brightnes_map.reserve(h);
        for y in 0usize..h {
            let mut row = Vec::new();
            row.reserve(w);

            for x in 0usize..w {
                let red = buffer[(y * 4usize * w) + (x * 4)];
                let green = buffer[(y * 4usize * w) + (x * 4 + 1)];
                let blue = buffer[(y * 4usize * w) + (x * 4 + 2)];

                let brightnes =
                    relative_brightnes(red as f64, green as f64, blue as f64);

                row.push(brightnes);
            }
            brightnes_map.push(row);
        }

        let _ = std::mem::replace(&mut self.brightnes_map, brightnes_map);

        yew_ctx.link().send_message(Msg::Render);
    }

    fn render(&mut self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        ctx.set_fill_style(&JsValue::from("rgb(73,79,92)"));
        ctx.fill_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

        let map = &self.brightnes_map;
        self.particles.iter_mut().for_each(|particle| {
            particle.update(map);
            ctx.set_global_alpha(particle.speed * 0.02);
            particle.render(&ctx);
        });

        window()
            .unwrap()
            .request_animation_frame(self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
}
