mod api;
mod auth;
mod components;
mod models;

use crate::api::fetch_active_sessions;
use crate::auth::{clear_token_from_storage, get_token_from_storage, save_token_to_storage};
use crate::components::active_sessions::ActiveSessions;
use crate::components::all_sessions::AllSessions;
use crate::components::evaluation::Evaluation;
use crate::components::login::Login;
use crate::components::start_session::StartSession;
use crate::components::stats::Stats;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    let (token, set_token) = create_signal(get_token_from_storage());
    let (view_mode, set_view_mode) = create_signal("timer");

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
                    let t_stored = store_value(t);
                    let sessions = create_resource(move || t_stored.get_value(), |t| fetch_active_sessions(t));

                    view! {
                        <div class="nav-tabs mb-1">
                            <button
                                class=move || if view_mode.get() == "timer" { "nav-btn active" } else { "nav-btn" }
                                on:click=move |_| set_view_mode.set("timer")
                            >
                                "Timer"
                            </button>
                            <button
                                class=move || if view_mode.get() == "stats" { "nav-btn active" } else { "nav-btn" }
                                on:click=move |_| set_view_mode.set("stats")
                            >
                                "Statistiken"
                            </button>
                            <button
                                class=move || if view_mode.get() == "evaluation" { "nav-btn active" } else { "nav-btn" }
                                on:click=move |_| set_view_mode.set("evaluation")
                            >
                                "Auswertung"
                            </button>
                        </div>

                        {move || match view_mode.get() {
                            "timer" => view! {
                                <section class="card">
                                    <StartSession
                                        token=t_stored.get_value()
                                        on_success=move |_| sessions.refetch()
                                    />
                                </section>
                                <section class="card">
                                    <ActiveSessions
                                        token=t_stored.get_value()
                                        sessions=sessions
                                    />
                                </section>
                                <section class="card">
                                    <AllSessions
                                        token=t_stored.get_value()
                                        on_update=move |_| sessions.refetch()
                                    />
                                </section>
                            }.into_view(),
                            "stats" => view! {
                                <section class="card">
                                    <Stats token=t_stored.get_value() />
                                </section>
                            }.into_view(),
                            "evaluation" => view! {
                                <section class="card">
                                    <Evaluation token=t_stored.get_value() />
                                </section>
                            }.into_view(),
                            _ => view! { <div/> }.into_view()
                        }}
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
