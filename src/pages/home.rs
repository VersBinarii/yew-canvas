use crate::components::{effects::ParticleMe, main_title::ParticleLogo};
use yew::prelude::*;

pub struct Home;

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _: &Context<Self>, _: Self::Message) -> bool {
        false
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <div id="spotlight" class="animated fadeIn">
              <div id="home-center">
                <ParticleLogo height={100} width={350} title={"Vers Binarii"} />
                <ParticleMe height={800} width={600}/>
              </div>
            </div>
        }
    }
}
