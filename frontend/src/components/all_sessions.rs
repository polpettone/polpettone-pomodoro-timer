use crate::api::{delete_session, fetch_all_sessions};
use crate::components::edit_session::EditSession;
use crate::models::Session;
use leptos::*;

#[component]
pub fn AllSessions(token: String, #[prop(into)] on_update: Callback<()>) -> impl IntoView {
    let token = store_value(token);
    let (editing_session, set_editing_session) = create_signal(None::<Session>);
    let (is_open, set_is_open) = create_signal(false);
    let sessions = create_resource(
        move || (is_open.get(), token.get_value()),
        |(open, t)| async move {
            if open {
                fetch_all_sessions(t).await
            } else {
                Ok(vec![])
            }
        },
    );

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
                                                        <th>"Aktion"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {data.into_iter().map(|s| {
                                                        let s_edit = s.clone();
                                                        view! {
                                                            <tr>
                                                                <td class="text-nowrap">{s.start}</td>
                                                                <td>{s.description}</td>
                                                                <td class="text-nowrap">{s.duration.secs / 60} " Min"</td>
                                                                <td><span class=format!("state-tag {}", s.state.to_lowercase())>{s.state}</span></td>
                                                                <td>
                                                                    <div class="flex gap-2">
                                                                        <button
                                                                            class="edit-btn"
                                                                            on:click=move |_| set_editing_session.set(Some(s_edit.clone()))
                                                                        >
                                                                            "Bearbeiten"
                                                                        </button>
                                                                        <button
                                                                            class="edit-btn"
                                                                            style="border-color: #f44336; color: #f44336;"
                                                                            on:click={
                                                                                let s_id = s.id;
                                                                                move |_| {
                                                                                    let t = token.get_value();
                                                                                    if window().confirm_with_message("Sitzung wirklich löschen?").unwrap_or(false) {
                                                                                        spawn_local(async move {
                                                                                            if let Ok(_) = delete_session(t, s_id).await {
                                                                                                sessions.refetch();
                                                                                                on_update.call(());
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                }
                                                                            }
                                                                        >
                                                                            "Löschen"
                                                                        </button>
                                                                    </div>
                                                                </td>
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

            {move || editing_session.get().map(|s| {
                let token_edit = token.get_value();
                view! {
                    <EditSession
                        token=token_edit
                        session=s
                        on_close=move |_| set_editing_session.set(None)
                        on_updated=move |_| {
                            sessions.refetch();
                            on_update.call(());
                        }
                    />
                }
            })}
        </div>
    }
}
