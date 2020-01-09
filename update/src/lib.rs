extern crate multiproof_rs;
extern crate rlp;

use multiproof_rs::NibbleKey;

// Where the new, updated root is stored.
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut newroot: &mut [u8] = &mut [0u8; 32];

fn update() -> Result<Vec<u8>, String> {
    Err("Not implemented")
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let root = update().unwrap();
    unsafe {
        newroot[..].copy_from_slice(&root[..]);
    }
}

#[cfg(test)]
mod tests {}
