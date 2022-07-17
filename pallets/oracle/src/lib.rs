#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[derive(Deserialize)]
struct Guild {
	id: u32,
	name: String,
	roles: Vec<GuildRole>,
}

#[derive(Deserialize)]
struct GuildRole {
	id: u32,
	members: Option<Vec<String>>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::{Guild, GuildRole};

	use frame_support::prelude::*;
	use frame_system::{offchain::Signer, prelude::*};
	use sp_runtime::offchain::http;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			log::info!("Hello World from offchain workers");
			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!("Current block: {:?}, parent hash: {:?}", block_number, parent_hash);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		fn query_guild_api() {}
	}

	impl<T: Config> Pallet<T> {
		fn query_guild_api_and_send_signed() -> Result<(), String> {}

		fn fetch_data() -> Result<String, http::Error> {
			let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2000));
			let request = http::Request::get("https://api.guild.xyz/v1/guild/8879")
				.deadline(deadline)
				.send()
				.map_err(|_| http::Error::IoError)?;
			let response =
				pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
			if response.code != 200 {
				log::error!("Unexpected status code: {}", response.code);
				return Err(http::Error::Unknown)
			}
			let body =
				sp_std::str::from_utf8(&response.body().collect::<Vec<u8>>()).map_err(|_| {
					log::error!("No UTF8 body");
					http::Error::Unknown
				})?;

			let guild = todo!("deserialize into guild");
			todo!();
		}
	}
}
