use gn_client::{
    tx::{Keypair, PairT},
    Api,
};

pub async fn rotate(api: Api) {
    let keys = api.rpc().rotate_keys().await.unwrap();
    println!("{keys:?}");
    println!("{}", keys.len());
    println!("{}", hex::encode(keys.0))
}

pub fn generate(curve: &str, password: Option<&str>) {
    match curve {
        "sr25519" => {
            let (keypair, phrase, seed) = Keypair::generate_with_phrase(password);
            println!("{}", keypair.public());
            println!("{phrase}");
            println!("{seed:?}");
        }
        _ => unimplemented!(),
    }
}
