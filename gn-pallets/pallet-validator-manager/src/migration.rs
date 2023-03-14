use super::*;

pub fn on_runtime_upgrade<T: Config>() {
    let validators = Validators::<T>::get();
    ApprovedValidators::<T>::set(validators);
}
