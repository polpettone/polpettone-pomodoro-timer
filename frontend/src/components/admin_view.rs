use crate::api::fetch_users;
use crate::models::User;
use leptos::*;

#[component]
pub fn AdminView(token: String) -> impl IntoView {
    let users_resource = create_resource(move || token.clone(), fetch_users);

    view! {
        <div class="admin-view">
            <h2>"Benutzerverwaltung (Admin)"</h2>

            <Transition fallback=move || view! { <p class="status-msg">"Lade Benutzerliste..."</p> }>
                {move || {
                    users_resource.get().map(|res| match res {
                        Ok(users) => {
                            view! {
                                <div class="table-container card">
                                    <table>
                                        <thead>
                                            <tr>
                                                <th>"ID"</th>
                                                <th>"Benutzername"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {users.into_iter().map(|user| {
                                                view! {
                                                    <tr>
                                                        <td class="text-nowrap" style="font-family: monospace; font-size: 0.8rem; color: #666;">
                                                            {user.id.to_string()}
                                                        </td>
                                                        <td style="font-weight: bold;">{user.username}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_view()
                        },
                        Err(e) => view! { <p class="error-msg">{e}</p> }.into_view()
                    })
                }}
            </Transition>
        </div>
    }
}
