use leptos::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    pub start: String,
    pub state: String,
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"Polpettone Pomodoro Timer"</h1>
            <section>
                <StartSession />
            </section>
            <hr />
            <section>
                <ActiveSessions />
            </section>
        </main>
    }
}

async fn fetch_active_sessions() -> Result<Vec<Session>, String> {
    let resp = Request::get("http://127.0.0.1:3000/sessions/active")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        return Err(format!("Fehler beim Laden: {}", resp.status()));
    }

    resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())
}

#[component]
fn ActiveSessions() -> impl IntoView {
    let sessions = create_resource(|| (), |_| fetch_active_sessions());

    view! {
        <div>
            <h2>"Aktive Sitzungen"</h2>
            <button on:click=move |_| sessions.refetch()>
                "↻ Aktualisieren"
            </button>
            <Transition fallback=move || view! { <p>"Lade..."</p> }>
                {move || {
                    sessions.get().map(|res| match res {
                        Ok(data) => {
                            if data.is_empty() {
                                view! { <p>"Keine aktiven Sitzungen vorhanden."</p> }.into_view()
                            } else {
                                view! {
                                    <ul>
                                        {data.into_iter().map(|s| {
                                            view! {
                                                <li>
                                                    <strong>{s.description}</strong>
                                                    " (" {s.duration.secs / 60} " Min) - Seit: " {s.start}
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_view()
                            }
                        },
                        Err(e) => view! { <p style="color: red">{e}</p> }.into_view(),
                    })
                }}
            </Transition>
        </div>
    }
}

#[component]
fn StartSession() -> impl IntoView {
    let (description, set_description) = create_signal("".to_string());
    let (duration, set_duration) = create_signal(25);

    let start_action = create_action(|(desc, dur): &(String, u64)| {
        let desc = desc.clone();
        let dur = *dur;
        async move {
            let payload = serde_json::json!({
                "description": desc,
                "duration_minutes": dur,
            });

            Request::post("http://127.0.0.1:3000/sessions/start")
                .json(&payload)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        }
    });

    view! {
        <div>
            <h2>"Neue Sitzung starten"</h2>
            <div style="display: flex; gap: 1rem; align-items: center;">
                <input 
                    type="text" 
                    placeholder="Beschreibung"
                    on:input=move |ev| set_description.set(event_target_value(&ev))
                    prop:value=description
                />
                <input 
                    type="number" 
                    on:input=move |ev| set_duration.set(event_target_value(&ev).parse().unwrap_or(25))
                    prop:value=duration
                />
                <button 
                    on:click=move |_| start_action.dispatch((description.get(), duration.get()))
                    disabled=move || start_action.pending().get()
                >
                    "Start"
                </button>
            </div>
            {move || {
                if let Some(res) = start_action.value().get() {
                    match res {
                        Ok(_) => view! { <p style="color: green">"Sitzung gestartet!"</p> }.into_view(),
                        Err(e) => view! { <p style="color: red">{e}</p> }.into_view(),
                    }
                } else {
                    view! { <span /> }.into_view()
                }
            }}
        </div>
    }
}
