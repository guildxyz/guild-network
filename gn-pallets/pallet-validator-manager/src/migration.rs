use super::*;

pub fn on_runtime_upgrade<T: Config>() {
    let validators = Validators::<T>::get();
    log::info!("# validators: {}", validators.len());
    log::info!(
        "# approved validators: {}",
        ApprovedValidators::<T>::get().len()
    );
    ApprovedValidators::<T>::set(validators);
    log::info!(
        "# approved validators post-migration: {}",
        ApprovedValidators::<T>::get().len()
    );
}
