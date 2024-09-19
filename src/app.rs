use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos_use::{use_websocket, UseWebSocketReturn};
use wasm_bindgen::{prelude::Closure, JsCast};
use std::{borrow::BorrowMut, str::FromStr, sync::{Arc, Mutex}, time::Duration};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize)]
struct Color(u8,u8,u8);

#[derive(Clone, Serialize, Deserialize)]
pub struct Client {
    name : String,
    color : Color,
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
const PING_INTERVAL : i32 = 200;


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
        return Err(ServerFnError::Response(String::from("owwwo you dwo nwot hwave an acwount :3")));
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
    let new_user = Client {
        name : String::from("bob dole"), 
        color : Color(1,1,1), 
        last_ping : SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64,
        uuid : uuid::Uuid::new_v4().to_string(),
        last_updated : 0};
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.clients.push(new_user.clone());
    Ok(new_user)
}

async fn delete_inactive_clients() {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let time_now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time travel was invented").as_millis() as u64;
    state.clients.retain(|client| time_now - client.last_ping < TIMEOUT_MS);
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
    let (peer_count, set_peer_count) = create_signal(0usize);
    let (client, set_client) = create_signal::<Option<Client>>(None);
    let fetch_state = move || {
        spawn_local( async move {
            if client.get_untracked().is_none() {
                return;
            }
            let state = fetch_global_state(client.get_untracked().unwrap().uuid.clone()).await.unwrap_or_else(|_| None);
            match state {
                Some(received_state) => {
                    let length = received_state.clients.len();
                    set_peer_count.update(|value| {*value = length});
                },
                None => ()
            }
        })
    };

    create_effect(move |_| {
        let fetch_state = fetch_state.clone();
        let window = window();
        let interval_closure = Closure::wrap(Box::new(move || {
            fetch_state();
        }) as Box<dyn Fn()>);
        window.set_interval_with_callback_and_timeout_and_arguments_0(interval_closure.as_ref().unchecked_ref(),PING_INTERVAL);

        interval_closure.forget();
    });

    create_effect(move |_| {
        spawn_local(async move {
            let client = register_user().await.unwrap();
            set_client.update(|value| *value = Some(client));
        });
    });

    view! {
        <h1>"Welcome, " {move || if client.get().is_some() {client.get().unwrap().name} else { String::from("uninitialized user") }} "!"</h1>
        <h2>{peer_count}</h2>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
