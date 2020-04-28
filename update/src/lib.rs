extern crate jupiter_account;
extern crate multiproof_rs;
extern crate rlp;

use jupiter_account::Account;
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
    let accounts: Vec<Account> =
        unsafe { rlp::decode_list::<Account>(&account_list[..account_list_size]) };

    // Deserialize the data into a tree
    let input_proof = unsafe { serialized_proof.to_vec() };
    let proof = rlp::decode::<Multiproof>(&input_proof).unwrap();
    let mut tree: Node = proof.rebuild()?;
    for account in accounts.iter() {
        match account {
            Account::Empty => panic!("Not supported in this version"),
            Account::Existing(addr, _, _, _, _) => {
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
    use super::*;
    use multiproof_rs::*;

    fn prepare_env(tree_keys: Vec<NibbleKey>, proof_keys: Vec<NibbleKey>, accounts: Vec<u8>) {
        let mut root = Node::default();
        for key in tree_keys.iter() {
            root.insert(key, vec![0u8; 32]).unwrap();
        }

        let proof = make_multiproof(&root, proof_keys).unwrap();
        let encoding = rlp::encode(&proof);

        unsafe {
            &mut serialized_proof[..].copy_from_slice(&[0u8; 1024]);
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        assert!(accounts.len() < unsafe { account_list.len() });
        unsafe {
            &mut account_list[..].copy_from_slice(&[0u8; 1024]);
            &mut account_list[..accounts.len()].copy_from_slice(&accounts[..]);
        };
    }

    #[test]
    fn test_update_normal() {
        let tree_keys = vec![
            NibbleKey::from(vec![0u8; 32]),
            NibbleKey::from(vec![1u8; 32]),
        ];

        let accounts = rlp::encode_list::<Account, Account>(&vec![Account::Existing(
            NibbleKey::from(vec![1u8; 32]),
            0,
            0,
            vec![],
            false,
        )]);

        prepare_env(tree_keys.clone(), tree_keys, accounts);
        assert_eq!(
            update().unwrap(),
            vec![
                170, 88, 142, 70, 168, 127, 29, 152, 75, 55, 157, 146, 147, 100, 73, 50, 155, 23,
                12, 124, 36, 168, 8, 31, 91, 26, 215, 247, 112, 139, 114, 173
            ]
        );
    }

    #[test]
    fn test_no_update() {
        let tree_keys = vec![
            NibbleKey::from(vec![0u8; 32]),
            NibbleKey::from(vec![1u8; 32]),
        ];

        prepare_env(tree_keys.clone(), tree_keys, vec![]);
        assert_eq!(
            update().unwrap(),
            vec![
                235, 187, 103, 171, 221, 121, 254, 77, 140, 151, 30, 221, 136, 176, 23, 212, 99,
                236, 120, 222, 139, 68, 122, 134, 96, 214, 189, 187, 175, 12, 197, 17
            ]
        );
    }

    #[test]
    fn test_update_missing() {
        let tree_keys = vec![
            NibbleKey::from(vec![0u8; 32]),
            NibbleKey::from(vec![1u8; 32]),
        ];

        let accounts = vec![Account::Existing(
            NibbleKey::from(vec![2u8; 32]),
            0,
            0,
            vec![],
            false,
        )];
        let accounts_rlp = rlp::encode_list::<Account, Account>(&accounts);

        let mut final_tree = Node::default();
        final_tree
            .insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        final_tree
            .insert(&NibbleKey::from(vec![1u8; 32]), vec![0u8; 32])
            .unwrap();
        final_tree
            .insert(&NibbleKey::from(vec![2u8; 32]), rlp::encode(&accounts[0]))
            .unwrap();

        prepare_env(tree_keys.clone(), tree_keys, accounts_rlp);
        assert_eq!(update().unwrap(), final_tree.hash());
    }
}
