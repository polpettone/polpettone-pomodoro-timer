use leptos::*;
use crate::api::fetch_all_sessions;

#[component]
pub fn AllSessions(token: String) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let sessions = create_resource(move || (is_open.get(), token.clone()), |(open, t)| async move {
        if open {
            fetch_all_sessions(t).await
        } else {
            Ok(vec![])
        }
    });

    view! {
        <div class="all-sessions">
            <button 
                class="expand-btn" 
                on:click=move |_| set_is_open.update(|v| *v = !*v)
            >
                {move || if is_open.get() { "Historie ausblenden ▲" } else { "Historie anzeigen ▼" }}
            </button>
            
            <div class="expand-content" class:open=is_open>
                <Transition fallback=move || view! { <p class="status-msg">"Lade Historie..."</p> }>
                    {move || {
                        sessions.get().map(|res| match res {
                            Ok(data) => {
                                if data.is_empty() {
                                    view! { <p class="status-msg">"Keine Sitzungen in der Historie."</p> }.into_view()
                                } else {
                                    view! {
                                        <div class="table-container">
                                            <table>
                                                <thead>
                                                    <tr>
                                                        <th>"Start"</th>
                                                        <th>"Beschreibung"</th>
                                                        <th>"Dauer"</th>
                                                        <th>"Status"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {data.into_iter().map(|s| {
                                                        view! {
                                                            <tr>
                                                                <td class="text-nowrap">{s.start}</td>
                                                                <td>{s.description}</td>
                                                                <td class="text-nowrap">{s.duration.secs / 60} " Min"</td>
                                                                <td><span class=format!("state-tag {}", s.state.to_lowercase())>{s.state}</span></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_view()
                                }
                            },
                            Err(e) => view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view(),
                        })
                    }}
                </Transition>
            </div>
        </div>
    }
}
