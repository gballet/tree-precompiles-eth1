extern crate multiproof_rs;
extern crate rlp;

#[allow(non_upper_case_globals)]
static mut input_data: &'static [u8] = &[0u8; 32];

extern "C" {
    static mut output: bool;
    static input_size: usize;
}

use multiproof_rs::{rebuild, Multiproof};

fn verify() -> Result<bool, String> {
    // Get the data
    unsafe {
        if input_size < input_data.len() {
            return Err(format!("input size exceeds allowed data"));
        }
    }
    let input = unsafe { input_data.to_vec() };

    // Deserialize the data into a tree
    let proof = rlp::decode::<Multiproof>(&input).unwrap();
    match rebuild(&proof) {
        Ok(_) => Ok(true),
        _ => Ok(false),
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let out = verify().unwrap();
    unsafe {
        output = out;
    }
}

#[cfg(test)]
mod tests {
    use super::multiproof_rs::node::*;
    use super::multiproof_rs::utils::*;
    use super::multiproof_rs::*;
    use super::*;

    #[test]
    fn test_recover_account() {
        let mut root = Node::default();
        insert_leaf(&mut root, &NibbleKey::from(vec![0u8; 32]), vec![0u8; 32]).unwrap();
        let encoding = rlp::encode(&root);
        // TODO copy input data
        unsafe { input_size = encoding.len() };
        verify().unwrap();
    }
}
