use leptos::{
    component,
    view,
    IntoView,
    SignalGet,
};
use leptos_router::Redirect;

use super::{
    expect_context,
    Context,
};

#[component]
pub fn IndexPage() -> impl IntoView {
    let Context {
        no_accounts_yet,
        is_logged_in,
        ..
    } = expect_context();

    view! {
        {move || {
            match (no_accounts_yet.get(), is_logged_in.get()) {
                (true, false) => {
                    view!{
                        <Redirect path="/register" />
                    }.into_view()
                }
                (false, false) => {
                    view! {
                        <Redirect path="/login" />
                    }.into_view()
                }
                (false, true) => {
                    view! {
                        <MainPage />
                    }.into_view()
                },
                _ => unreachable!()
            }
        }}
    }
}

#[component]
pub fn MainPage() -> impl IntoView {
    view! {
        <div class="p-4">
            <h4>"TODO"</h4>
        </div>
    }
}
