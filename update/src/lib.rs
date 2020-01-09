extern crate multiproof_rs;
extern crate rlp;

use account;
use account::Account;
use multiproof_rs::{Multiproof, Node, ProofToTree, Tree};

// The RLP-serialized proof
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut serialized_proof: &mut [u8] = &mut [0u8; 1024];

// An RLP-encoded list of accounts to be verified
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut account_list: &mut [u8] = &mut [0u8; 1024];

// Where the new, updated root is stored.
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut newroot: &mut [u8] = &mut [0u8; 32];

fn rlp_stream_size(payload: Vec<u8>) -> usize {
    if payload.len() < 2 {
        return payload.len();
    }
    match payload[0] as usize {
        id if id < 192 => id,
        id if id < 247 => id - 192 + 1,
        id => {
            let size_size = id - 247;
            if id < size_size + 1 {
                panic!("Invalid payload");
            }
            let mut size: usize = 0;
            for i in 0..size_size {
                size = (size << 8) + payload[1 + i] as usize;
            }
            size + 1 + size_size
        }
    }
}

fn update() -> Result<Vec<u8>, String> {
    let account_list_size = unsafe { rlp_stream_size(account_list.to_vec()) };

    // Deserialize the accounts to verify
    let accounts: Vec<account::Account> =
        unsafe { rlp::decode_list::<Account>(&account_list[..account_list_size]) };

    // Deserialize the data into a tree
    let input_proof = unsafe { serialized_proof.to_vec() };
    let proof = rlp::decode::<Multiproof>(&input_proof).unwrap();
    let mut tree: Node = proof.rebuild()?;
    for account in accounts.iter() {
        match account {
            Account::Empty => panic!("Not supported in this version"),
            Account::Existing(addr, _, _, _) => {
                // XXX for this to work, multiproof-rs needs to be modified
                // to be able to overwrite keys, and Account needs to have
                // its first member as a public address.
                tree.insert(addr, rlp::encode(account))?;
            }
        }
    }
    Ok(tree.hash())
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
mod tests {
    use super::multiproof_rs::*;
    use super::*;

    #[test]
    fn normal_update() {
        let mut root = Node::default();
        root.insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        root.insert(&NibbleKey::from(vec![1u8; 32]), vec![1u8; 32])
            .unwrap();
        let proof = make_multiproof(
            &root,
            vec![
                NibbleKey::from(vec![1u8; 32]),
                NibbleKey::from(vec![2u8; 32]),
            ],
        )
        .unwrap();
        let encoding = rlp::encode(&proof);
        assert!(encoding.len() < unsafe { serialized_proof.len() });
        unsafe {
            serialized_proof[..].copy_from_slice(&[0u8; 1024]);
            serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        // Change 0x111..111 and add 0x222...222
        let keys = rlp::encode_list(&vec![
            Account::Existing(NibbleKey::from(vec![1u8; 32]), 0, vec![], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![], false),
        ]);
        assert!(keys.len() < unsafe { account_list.len() });
        unsafe {
            account_list[..].copy_from_slice(&[0u8; 1024]);
            account_list[..keys.len()].copy_from_slice(&keys);
        };

        let h = update().unwrap();
        assert_eq!(
            h,
            vec![
                148, 48, 108, 190, 102, 14, 113, 111, 29, 181, 184, 140, 248, 25, 58, 129, 200,
                126, 242, 57, 171, 233, 186, 169, 100, 79, 63, 231, 75, 131, 243, 157
            ]
        );
    }
}
