extern crate multiproof_rs;

use multiproof_rs::Multiproof;

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    // Get the data

    // Deserialize the data into a tree
    let proof = rlp::decode::<Multiproof>();
    let verified = match rebuild(&proof) {
        Ok(_) => true,
        None => false
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_recover_account() {}
}
