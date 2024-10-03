use html::{AnyElement, ElementDescriptor, Input};
use leptos::*;
use leptos_dom::{Element, IntoFragment};
use leptos_meta::*;
use leptos_router::*;
use rand::{seq::SliceRandom};
use wasm_bindgen::{prelude::Closure, JsCast};
use std::{str::FromStr, sync::{Arc, Mutex}, time::Duration};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

// #[derive(Clone, Serialize, Deserialize)]
// struct Color(u8,u8,u8);

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    name : String,
    color : u16,
    last_ping : u64,
    uuid : String,
    last_updated : u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GlobalState {
    clients : Vec<Client>,
    last_updated : u64,
}

impl FromStr for GlobalState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).unwrap_or_else(|_| return Err(String::from("Failed to parse GlobalState from JSON")))
    }
}

impl ToString for GlobalState {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

static GLOBAL_STATE : Lazy<Arc<Mutex<GlobalState>>> = Lazy::new(|| Arc::new(Mutex::new(GlobalState{
    clients: Vec::new(), 
    last_updated : SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64})));

static LAST_CHECK : Lazy<Arc<Mutex<u128>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

const TIMEOUT_MS : u64 = 2000;
const CHECK_TIMEOUT_EVERY_MS : u128 = 4000;
const PING_INTERVAL : i32 = 35;//was 200
const NOT_REGISTERED_ERROR : &str = "owwwo you dwo nwot hwave an acwount :3";
const ATTACK_COOLDOWN_MS : u32 = 5000;
const ATTACK_COOLDOWN_ERR_MARGIN_MS : u32 = 200;


#[server]
async fn fetch_global_state(uuid : String) -> Result<Option<GlobalState>,ServerFnError> {
    // #[cfg(feature = "ssr")] //holy grail
    // {
    //     actix_web::rt::time::sleep(Duration::from_millis(3000)).await;
    // }

    let mut last_check = LAST_CHECK.lock().unwrap();
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    if time_now - *last_check > CHECK_TIMEOUT_EVERY_MS {
        *last_check = time_now;
        delete_inactive_clients().await;
    }

    let mut state = GLOBAL_STATE.lock().unwrap();
    let mut client_search_result = state.clients.iter_mut().find(|value| value.uuid == uuid);
    if client_search_result.is_none() {
        return Err(ServerFnError::MissingArg(String::from(NOT_REGISTERED_ERROR)));
    }

    let client = client_search_result.unwrap();
    client.last_ping = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64;
    if client.last_updated == state.last_updated {
        return Ok(None);
    }
    Ok(Some(state.clone()))
}

#[server]
async fn register_user() -> Result<Client, ServerFnError> {
    let name = create_name().await.unwrap_or_else(|_| String::from("IO Error :D"));

    let new_user = Client {
        name, 
        // color : Color(1,1,1), 
        color : 0u16,
        last_ping : SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64,
        uuid : uuid::Uuid::new_v4().to_string(),
        last_updated : 0};
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.clients.push(new_user.clone());
    Ok(new_user)
}

#[server]
async fn update_color(uuid : String, color : u16) -> Result<(), ServerFnError> {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let mut client_search_result = state.clients.iter_mut().find(|value| value.uuid == uuid);
    if client_search_result.is_none() {
        return Err(ServerFnError::MissingArg(String::from("owwwo you dwo nwot hwave an acwount :3")));
    }

    let client = client_search_result.unwrap();
    client.last_ping = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64;
    client.color = color;
    Ok(())
}

async fn create_name() -> Result<String, std::io::Error> {
    let mut nouns : Vec<String> = std::fs::read_to_string("nouns.csv")?.split("\r\n").map(|str| String::from(str)).collect();
    let mut adjectives : Vec<String> = std::fs::read_to_string("adjectives.csv")?.split("\r\n").map(|str| String::from(str)).collect();
    
    let mut thread_rng = rand::thread_rng();

    nouns.shuffle(&mut thread_rng);
    adjectives.shuffle(&mut thread_rng);

    let name = adjectives[0].to_owned() + " " + &nouns[0];

    Ok(name)
}

async fn delete_inactive_clients() {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64;
    state.clients.retain(|client| time_now - client.last_ping < TIMEOUT_MS);
}

fn hue_to_hex(hue : u16) -> String {
    let mut hh : f64 = hue as f64;
    
    if hh > 360f64 {
        hh = 0f64;
    }
    hh = hh/60f64;

    let i = hh as i64;
    let ff : f64 = hh - i as f64;
    let p : f64 = 0f64;
    let q = 1.0 - ff;//t == ff
    let rgb : (f64,f64,f64) = match i {
        0 => (1.0f64, ff, p),
        1 => (q, 1.0f64, p),
        2 => (p, 1.0f64, ff),
        3 => (p, q, 1.0f64),
        4 => (ff, p, 1.0f64),
        _ => (1.0f64, p, q), 
    };
    let rgb : (u8, u8, u8) = ((rgb.0*255.0f64) as u8, (rgb.1*255.0f64) as u8, (rgb.2*255.0f64) as u8);
    String::from(format!("#{:02X?}{:02X?}{:02X?}", rgb.0, rgb.1, rgb.2))
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/anglu-website.css"/>
        <Title text="Academic English"/>
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (peers, set_peers) = create_signal::<Vec<Client>>(Vec::new());
    let (client, set_client) = create_signal::<Option<Client>>(None);
    let (color, set_color) = create_signal(0u16);
    let (string_color, set_string_color) = create_signal(String::new());
    let mut uuid = Option::<String>::from(None);
    // let mut user_requested : bool = false;

    let fetch_state = create_action(move |_: &()| {
        async move {
            if client.get_untracked().is_none() {
                let client = register_user().await;
                match client {
                    Ok(client) => set_client.update(|value| *value = Some(client)),
                    Err(e) => leptos::logging::log!("{}", e.to_string()),
                }
            }
            let state = fetch_global_state(client.get_untracked().unwrap().uuid.clone()).await;
            match state {
                Ok(option) => match option {
                    Some(received_state) => {
                    set_peers.update(|value| {*value = received_state.clients});
                    },
                    None => ()
                },
                Err(e) => {
                    if e.to_string() == "missing argument ".to_owned()+NOT_REGISTERED_ERROR {
                        // if !register.pending().get() {
                        //     register.dispatch(());
                        // }
                        let client = register_user().await;
                        match client {
                            Ok(client) => set_client.update(|value| *value = Some(client)),
                            Err(e) => leptos::logging::log!("{}", e.to_string()),
                        }
                    }
                },
            }
        }
    });


    let tick = move || {
        if fetch_state.pending().get() {
            return;
        }
        fetch_state.dispatch(());
    };

    let send_color_update = move || { spawn_local(async move {
            if client.get_untracked().is_none() {
                return;
            }
            let _ = update_color(client.get_untracked().unwrap().uuid.clone(),color.get_untracked()).await;
        })
    };

    create_effect(move |_| {
        let tick = tick.clone();
        let window = window();
        let interval_closure = Closure::wrap(Box::new(move || {
            tick();
        }) as Box<dyn Fn()>);
        let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(interval_closure.as_ref().unchecked_ref(),PING_INTERVAL);

        interval_closure.forget();
    });

    let mut cooldown_timer : i32 = 0;

    let tick_cooldown = move || {
        let parents = document().get_elements_by_class_name("grid-element-parent");

        for x in 0..parents.length() {
            let mut element = parents.item(x);
            if element.is_none() {
                continue;
            }
            let mut element = element.unwrap();
            element.set_attribute("style", "")
        }
    };

    let cooldown_animation = move || {
        let tick_cooldown = tick_cooldown.clone();
        let window = window();
        let interval_closure = Closure::wrap(Box::new(move || {
            tick_cooldown();
        }) as Box<dyn Fn()>);
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(interval_closure.as_ref().unchecked_ref(),PING_INTERVAL);

        interval_closure.forget();
    };

    create_effect(move |_| {
        set_string_color.set(hue_to_hex(color.get()));
    });

    view! {{move || if client.get().is_some() {
            view! {
                <div class="header-container">
                    <h1>"Welcome, " {client.get().unwrap().name} "!"</h1>
                    <h2 style = {move || "color:".to_owned()+&string_color.get()+"!important"}>Pick your color:</h2>
                    <input
                        on:input = move |event| {send_color_update();set_color.set(event_target_value(&event).parse::<u16>().unwrap());}
                        type="range"
                        min = "0"
                        max = "359"
                        value = "0"
                        class = "slider"
                        ondragstart = "return false;"
                    />
                </div>
                <div class="gridcontainer">
                    <For
                        each = move || peers.get()
                        key = |peer| peer.uuid.clone()+&peer.color.to_string()
                        children = move |peer : Client| {
                            let splits : Vec<String> = peer.name.split(" ").map(|strindge| String::from(strindge)).collect();
                            view! {
                                <div class="grid-element-parent">
                                    <h1 class="gridelement" style = {move || "background-color:".to_owned()+&hue_to_hex(peer.color)+"!important"}>{&splits[0]}<br/>{&splits[1]}</h1>
                                </div>
                            }
                        }
                        />
                </div>
            }} else { view! {"" ""}
        }}
    }
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
