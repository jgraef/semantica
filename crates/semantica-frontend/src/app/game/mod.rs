use std::collections::HashMap;

use leptos::{
    component,
    create_rw_signal,
    expect_context,
    update,
    view,
    with,
    For,
    IntoView,
    RwSignal,
};
use semantica_protocol::spell::{
    ResponseSpellAmount,
    SpellId,
};

use crate::{
    app::{
        BootstrapIcon,
        Context,
    },
    error::Error,
    utils::spawn_local_and_handle_error,
};

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub spells: HashMap<SpellId, ResponseSpellAmount>,
    pub spells_sorted: Vec<SpellId>,
}

#[derive(Copy, Clone, Debug)]
struct GameState {
    pub inventory: RwSignal<Inventory>,
}

fn provide_game_state() -> GameState {
    let game_state = GameState {
        inventory: create_rw_signal(Default::default()),
    };

    leptos::provide_context(game_state.clone());

    game_state
}

#[component]
pub fn MainPage() -> impl IntoView {
    let Context { client, .. } = expect_context();
    let GameState { inventory, .. } = provide_game_state();

    spawn_local_and_handle_error(async move {
        let response = client.inventory().await?;

        let spells: HashMap<SpellId, ResponseSpellAmount> = response
            .inventory
            .into_iter()
            .map(|spell_amount| (spell_amount.spell.spell_id, spell_amount))
            .collect();

        let mut spells_sorted = spells
            .iter()
            .map(|(spell_id, spell_amount)| (spell_id, &spell_amount.spell.name))
            .collect::<Vec<_>>();
        spells_sorted.sort_by_cached_key(|(_, name)| name.to_lowercase());
        let spells_sorted = spells_sorted
            .into_iter()
            .map(|(spell_id, _)| *spell_id)
            .collect();

        update!(|inventory| {
            inventory.spells = spells;
            inventory.spells_sorted = spells_sorted;
        });

        Ok::<(), Error>(())
    });

    view! {
        <div class="d-flex flex-row h-100">
            <div class="d-flex flex-grow-1">
                <h4>"TODO"</h4>
            </div>
            <div class="d-flex flex-column w-25 h-100 border-start">
                <form class="position-relative">
                    <input
                        class="form-control"
                        type="text"
                        placeholder="Search"
                        aria-label="Search inventory"
                        style="--bs-border-radius: 0"
                    />
                    <div class="position-absolute top-50 end-0 translate-middle-y">
                        <span class="me-3"><BootstrapIcon icon="search" /></span>
                    </div>
                </form>
                <div class="flex-grow-1 p-2">
                    <For
                        each=move || with!(|inventory| inventory.spells_sorted.clone())
                        key=|id| *id
                        children=move |id| {
                            with!(|inventory| {
                                let spell_amount = inventory.spells.get(&id).unwrap();
                                view!{
                                    <div class="d-inline rounded bg-secondary py-1 px-2">
                                        {spell_amount.amount}
                                        <BootstrapIcon icon="x" />
                                        {spell_amount.spell.emoji.clone()}
                                        {spell_amount.spell.name.clone()}
                                    </div>
                                }
                            })
                        }
                    />
                </div>
            </div>
        </div>
    }
}
