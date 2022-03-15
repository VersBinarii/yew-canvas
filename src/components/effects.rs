use crate::fetch::fetch_image;
use rand::{thread_rng, Rng};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    window, CanvasRenderingContext2d, HtmlCanvasElement, ImageBitmap,
};
use yew::prelude::*;

const PIX2: f64 = std::f64::consts::PI * 2.0;

struct Particle {
    x: f64,
    y: f64,
    velocity: f64,
    size: f32,
    speed: f64,
    max_height: usize,
}

impl Particle {
    fn new(canvas_width: usize, canvas_height: usize) -> Self {
        let mut rng = thread_rng();
        let x = rng.gen_range(0f64..canvas_width as f64);
        let y = 0.0;
        let velocity = rng.gen_range(0.0..0.5);
        let size: f32 = rng.gen_range(0.0..1.0) + 1.0;
        Self {
            x,
            y,
            velocity,
            size,
            speed: 0.0,
            max_height: canvas_height,
        }
    }

    fn update(&mut self, map: &[Vec<f64>]) {
        let posx = self.x as usize;
        let posy = self.y as usize;
        self.speed = map[posy][posx];

        let movement = (2.5 - self.speed) + self.velocity;
        self.y += movement;
        if self.y >= self.max_height as f64 {
            self.y = 0f64;
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
    js_sys::Math::sqrt((r * r) * 0.229 + (g * g) * 0.587 + (b * b) * 0.114)
        / 100.0
}

pub enum Msg {
    Render,
    FetchOk(ImageBitmap),
    FetchFail(String),
    ImageDataAvailable(Vec<u8>),
}

#[derive(Properties, PartialEq, Copy, Clone)]
pub struct CanvasProps {
    pub width: usize,
    pub height: usize,
}

pub struct ParticleMe {
    canvas: NodeRef,
    particles: Vec<Particle>,
    cb: Closure<dyn FnMut()>,
    brightnes_map: Vec<Vec<f64>>,
}

impl Component for ParticleMe {
    type Message = Msg;
    type Properties = CanvasProps;
    fn create(ctx: &Context<Self>) -> Self {
        let width = ctx.props().width;
        let height = ctx.props().height;
        let particles =
            (0..8000).map(|_| Particle::new(width, height)).collect();
        let l = ctx.link().clone();
        let cb =
            Closure::wrap(Box::new(move || l.send_message(Msg::Render))
                as Box<dyn FnMut()>);
        ctx.link().send_future(async {
            match fetch_image("content/GutsBerserk.png").await {
                Ok(image) => Msg::FetchOk(image),
                Err(err) => {
                    Msg::FetchFail(err.to_string())
                }
            }
        });
        Self {
            particles,
            canvas: NodeRef::default(),
            cb,
            brightnes_map: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let h = ctx.props().height;
        let w = ctx.props().width;
        match msg {
            Msg::Render => {
                self.render();
                false
            }
            Msg::FetchFail(e) => {
                gloo::console::log!(&format!("{:?}", &e));
                false
            }
            Msg::FetchOk(image_bitmap) => {
                let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
                let cctx: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();

                cctx.draw_image_with_image_bitmap(&image_bitmap, 0.0, 0.0)
                    .unwrap();

                let image_data =
                    cctx.get_image_data(0.0, 0.0, w as f64, h as f64).unwrap();

                cctx.clear_rect(0.0, 0.0, w as f64, h as f64);
                ctx.link().send_message(Msg::ImageDataAvailable(
                    (*image_data.data()).clone(),
                ));
                false
            }
            Msg::ImageDataAvailable(buffer) => {
                let mut brightnes_map = Vec::new();
                brightnes_map.reserve(h);
                for y in 0usize..h {
                    let mut row = Vec::new();
                    row.reserve(w);

                    for x in 0usize..w {
                        let red = buffer[(y * 4usize * w) + (x * 4)];
                        let green = buffer[(y * 4usize * w) + (x * 4 + 1)];
                        let blue = buffer[(y * 4usize * w) + (x * 4 + 2)];

                        let brightnes = relative_brightnes(
                            red as f64,
                            green as f64,
                            blue as f64,
                        );

                        row.push(brightnes);
                    }
                    brightnes_map.push(row);
                }

                let _ =
                    std::mem::replace(&mut self.brightnes_map, brightnes_map);

                ctx.link().send_message(Msg::Render);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
            <canvas
                id="canvas-img"
                width={ctx.props().width.to_string()}
                height={ctx.props().height.to_string()}
                ref={self.canvas.clone()}>
                </canvas>
            </div>
        }
    }
}

impl ParticleMe {
    fn render(&mut self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        ctx.set_global_alpha(0.05);
        ctx.set_fill_style(&JsValue::from("rgb(0,0,0)"));
        ctx.fill_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
        ctx.set_global_alpha(0.2);

        let map = &self.brightnes_map;
        self.particles.iter_mut().for_each(|particle| {
            particle.update(map);
            ctx.set_global_alpha(particle.speed * 0.10);
            particle.render(&ctx);
        });

        window()
            .unwrap()
            .request_animation_frame(self.cb.as_ref().unchecked_ref())
            .unwrap();
    }
}
