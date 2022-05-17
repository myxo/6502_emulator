#[cfg(target_family = "wasm")]
mod debugger_client {
use yew::prelude::*;

#[derive(serde::Deserialize, Debug, PartialEq)]
struct CpuState{
    x: u8,
    y: u8,
    a: u8,
}

enum Msg {
    AddOne,
    CpuState(CpuState),
}

struct Model {
    value: i64,
    cpu_state: CpuState,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            value: 0,
            cpu_state: CpuState { x:0, y:0, a:0 },
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        println!("This is update call");
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            },
            Msg::CpuState(cpu_state) => {
                if self.cpu_state != cpu_state {
                    self.cpu_state = cpu_state;
                    self.value += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        println!("This is view call");
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let vec = (0..=100).collect::<Vec<_>>();

        let link = ctx.link().clone();
        //let resp = reqwest::blocking::get("http://127.0.0.1:7878").unwrap().text().unwrap();
        wasm_bindgen_futures::spawn_local(async move {
                let fetched_asm : CpuState = reqwasm::http::Request::get("http://127.0.0.1:7878")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                link.send_message(Msg::CpuState(fetched_asm));
        });

        let link = ctx.link().clone();
        html! {
            <>
            <div> <p>{ format!("{:?}", self.cpu_state) }</p> </div>
            <div style="overflow-y: scroll; height:400px; width:200px">
                { 
                    vec.into_iter().map(|name| { html!{ <div> {name}</div>} }).collect::<Html>()
                }
            </div>
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
            </>
        }
    }
}

pub fn run() {
    yew::start_app::<Model>();
}

}

#[cfg(target_family = "wasm")]
fn main() {
    debugger_client::run();
}


#[cfg(not(target_family = "wasm"))]
fn main() {
    panic!("This program is only intended to run on WASM target.");
}
