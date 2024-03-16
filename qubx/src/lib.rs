mod qubx;
mod qlist;
mod qubx_components;
mod qubx_common;
mod qubx_pmanage;

pub use qubx::Qubx;
pub use qubx_common::StreamParameters;
pub use qubx_components::*;


#[cfg(test)]
mod test {

    use super::Qubx;

    #[test]
    fn get_devices() {
        Qubx::get_devices_info();
    }
    
}