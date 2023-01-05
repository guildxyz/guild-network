use gloo::events::EventListener;
use gloo::utils::window;
use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use web_sys::{Event, EventTarget};
use yew::prelude::*;

fn get_initial_state() -> Object {
    let initial_state = Object::new();

    Reflect::set(&initial_state, &"address".into(), &JsValue::UNDEFINED).unwrap();
    Reflect::set(&initial_state, &"connector".into(), &JsValue::UNDEFINED).unwrap();
    Reflect::set(&initial_state, &"isConnected".into(), &false.into()).unwrap();
    Reflect::set(&initial_state, &"isConnecting".into(), &false.into()).unwrap();
    Reflect::set(&initial_state, &"isDisconnected".into(), &true.into()).unwrap();
    Reflect::set(&initial_state, &"isReconnecting".into(), &false.into()).unwrap();
    Reflect::set(&initial_state, &"status".into(), &"disconnected".into()).unwrap();

    initial_state
}

#[hook]
pub fn use_ens_account() -> UseStateHandle<Object> {
    let account_state = use_state(get_initial_state);

    {
        let account_state = account_state.clone();
        use_effect(|| {
            let listener = EventListener::new(&window(), "ACCOUNT_STATE_CHANGE", move |_| {
                let window: Object = window().into();
                let new_account_state: Object = Reflect::get(&window, &"accountState".into())
                    .unwrap()
                    .into();

                account_state.set(new_account_state)
            });

            move || {
                drop(listener);
            }
        });
    }

    account_state
}
