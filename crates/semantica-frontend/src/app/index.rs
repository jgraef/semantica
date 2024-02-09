use std::collections::HashMap;

use leptos::{
    component,
    spawn_local,
    view,
    IntoView,
    RwSignal,
    SignalGet,
};
use leptos_router::Redirect;
use semantica_protocol::spell::{
    Spell,
    SpellId,
};

use super::{
    expect_context,
    game::MainPage,
    BootstrapIcon,
    Context,
};
use crate::error::Error;

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
