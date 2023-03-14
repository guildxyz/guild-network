use super::*;

pub fn on_runtime_upgrade<T: Config>() {
    let validators = Validators::<T>::get();
    log::info!("validators: {:?}", validators);
    log::info!("approved validators: {:?}", ApprovedValidators::<T>::get());
    ApprovedValidators::<T>::set(validators);
    log::info!(
        "newly approved validators: {:?}",
        ApprovedValidators::<T>::get()
    );
}
