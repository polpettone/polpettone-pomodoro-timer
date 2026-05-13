use leptos::*;
use crate::models::Session;
use crate::components::timer::Timer;

#[component]
pub fn ActiveSessions(sessions: Resource<String, Result<Vec<Session>, String>>) -> impl IntoView {
    view! {
        <div>
            <h2>"Aktive Sitzungen"</h2>
            <Transition fallback=move || view! { <p class="status-msg">"Lade..."</p> }>
                {move || {
                    sessions.get().map(|res| match res {
                        Ok(data) => {
                            if data.is_empty() {
                                view! { <p class="status-msg">"Keine aktiven Sitzungen vorhanden."</p> }.into_view()
                            } else {
                                view! {
                                    <ul>
                                        {data.into_iter().map(|s| {
                                            view! {
                                                <li>
                                                    <strong>{s.description.clone()}</strong>
                                                    " - " <Timer session=s />
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_view()
                            }
                        },
                        Err(e) => view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view(),
                    })
                }}
            </Transition>
        </div>
    }
}
