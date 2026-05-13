mod models;
mod api;
mod auth;
mod components;

use leptos::*;
use crate::api::fetch_active_sessions;
use crate::auth::{get_token_from_storage, save_token_to_storage, clear_token_from_storage};
use crate::components::active_sessions::ActiveSessions;
use crate::components::all_sessions::AllSessions;
use crate::components::start_session::StartSession;
use crate::components::login::Login;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    let (token, set_token) = create_signal(get_token_from_storage());

    let logout = move |_| {
        clear_token_from_storage();
        set_token.set(None);
    };

    view! {
        <main class="container">
            <div class="header-flex">
                <h1>"Polpettone Pomodoro Timer"</h1>
                {move || token.get().is_some().then(|| view! {
                    <button class="logout-btn" on:click=logout>"Abmelden"</button>
                })}
            </div>
            
            {move || match token.get() {
                Some(t) => {
                    let t_active = t.clone();
                    let t_start = t.clone();
                    let t_all = t.clone();
                    let sessions = create_resource(move || t_active.clone(), |t| fetch_active_sessions(t));
                    view! {
                        <section class="card">
                            <StartSession token=t_start on_success=move |_| sessions.refetch() />
                        </section>
                        <section class="card">
                            <ActiveSessions sessions=sessions />
                        </section>
                        <section class="card">
                            <AllSessions token=t_all />
                        </section>
                    }.into_view()
                },
                None => view! {
                    <section class="card">
                        <Login on_login=move |new_token: String| {
                            save_token_to_storage(&new_token);
                            set_token.set(Some(new_token));
                        }/>
                    </section>
                }.into_view()
            }}
        </main>
    }
}
