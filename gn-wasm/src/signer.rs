// TODO WIP
use js_sys::{Error, Function, JsString, Object, Promise};
use web_sys::{console, window};
use std::str::FromStr;

fn get_sign_request(msg: &str, address: &str) -> Result<js_sys::Object, Error> {
    let sign_request_param = js_sys::Object::new();

    js_sys::Reflect::set(
        &sign_request_param,
        &JsString::from("address"),
        &address.into(),
    )?;
    js_sys::Reflect::set(&sign_request_param, &JsString::from("data"), &msg.into())?;

    Ok(sign_request_param)
}

pub struct WasmSigner {
    pub lib: Object,
    pub account_id: AccountId,
    pub address: SubstrateAddress,
}

impl WasmSigner {
    pub async fn new() -> Result<Self, Error> {
        let window = window().expect("Failed to access window object");

        let injected_web3 = window
            .get("injectedWeb3")
            .expect("Failed to access window.injectedWeb3");

        let polkadot_js: JsValue =
            js_sys::Reflect::get(&injected_web3, &JsString::from("polkadot-js"))?;

        let enable: Function =
            js_sys::Reflect::get(&polkadot_js, &JsString::from("enable"))?.into();

        let lib: Object =
            wasm_bindgen_futures::JsFuture::from(Promise::from(enable.call0(&JsValue::NULL)?))
                .await?
                .into();

        let addresses: js_sys::Array = wasm_bindgen_futures::JsFuture::from(Promise::from(
            js_sys::Function::from(js_sys::Reflect::get(
                &js_sys::Reflect::get(&lib, &JsString::from("accounts"))?,
                &JsString::from("get"),
            )?)
            .call0(&JsValue::NULL)?,
        ))
        .await?
        .into();

        let name: String = js_sys::Reflect::get(&addresses.at(0), &"name".into())?
            .as_string()
            .expect("Failed to cast addresses[0] to String");
        let address: String = js_sys::Reflect::get(&addresses.at(0), &"address".into())?
            .as_string()
            .expect("Failed to cast addresses[0] to String");

        console::log_1(&format!("Hello {}! ({})", name, address).into());

        let account_id = AccountId::from_str(&address).expect("invalid address");
        let address = SubstrateAddress::from(account_id.clone());

        Ok(Self {
            lib,
            account_id, 
            address,
        })
    }
}

impl TxSignerTrait<ClientConfig> for WasmSigner {
    fn nonce(&self) -> Option<Index> {
        Some(123)
    }

    fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    fn address(&self) -> SubstrateAddress {
        self.address
    }

    fn sign(&self, signer_payload: &[u8]) -> Signature {
        let signer =
            js_sys::Reflect::get(&self.lib, &"signer".into()).expect("failed to get signer");
        let sign_raw: Function = js_sys::Reflect::get(&signer, &"signRaw".into())
            .expect("failed to get signature function")
            .into();

        let sign_payload =
            get_sign_request(signer_payload, &self.address).expect("failed to get sign request");

        sign_raw
            .call1(&JsValue::NULL, &sign_payload)
            .expect("failed to get sign promise")
            .into()
    }
}

#[wasm_bindgen(js_name = "polkadotSign")]
pub async fn polkadot_sign(message: &str) -> Result<(), Error> {
    console::log_2(&"Message to sign:".into(), &message.into());

    let signer = WasmSigner::new().await?;
    let signature = signer.sign(message.as_bytes());

    console::log_1(&signature);
    Ok(())
}
