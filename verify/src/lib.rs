extern crate multiproof_rs;
extern crate rlp;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut input_data: &'static mut [u8] = &mut [0u8; 1024];

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut output: bool = false;

#[allow(non_upper_case_globals)]
#[cfg(not(test))]
#[no_mangle]
static input_size: usize = 0usize;

#[allow(non_upper_case_globals)]
#[cfg(test)]
static mut input_size: usize = 0usize;

use multiproof_rs::{Multiproof, Node, ProofToTree};

fn verify() -> Result<bool, String> {
    // Get the data
    unsafe {
        if input_size > input_data.len() {
            return Err(format!("input size exceeds allowed data"));
        }
    }
    let input = unsafe { input_data.to_vec() };

    // Deserialize the data into a tree
    let proof = rlp::decode::<Multiproof>(&input).unwrap();
    match proof.rebuild() as Result<Node, String> {
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
        root.insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        root.insert(&NibbleKey::from(vec![1u8; 32]), vec![1u8; 32])
            .unwrap();
        let proof = make_multiproof(&root, vec![NibbleKey::from(vec![1u8; 32])]).unwrap();
        let encoding = rlp::encode(&proof);
        unsafe {
            input_size = encoding.len();
            &mut input_data[..input_size].copy_from_slice(&encoding);
        };

        // Verify that the acount was indeed in the proof
        assert!(verify().unwrap());
    }
}
