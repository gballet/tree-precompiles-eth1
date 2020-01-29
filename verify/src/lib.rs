extern crate multiproof_rs;
extern crate rlp;

use account;
use account::Account;

// The RLP-serialized proof
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut serialized_proof: &mut [u8] = &mut [0u8; 1024];

// An RLP-encoded list of accounts to be verified
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut address_list: &mut [u8] = &mut [0u8; 1024];

// An indicator whether the verification succeeded or not
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut valid: &mut [bool] = &mut [false; 1024];

use multiproof_rs::{Multiproof, Node, ProofToTree, Tree};

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

fn verify() -> Result<Vec<bool>, String> {
    let address_list_size = unsafe { rlp_stream_size(address_list.to_vec()) };
    // Debug traces, do not remove yet
    //unsafe {
    //println!(
    //"RLP to verify: {:?} {:?}",
    //&address_list[..address_list_size],
    //address_list_size
    //);
    //}
    // Deserialize the accounts to verify
    let accounts: Vec<Account> =
        unsafe { rlp::decode_list::<Account>(&address_list[..address_list_size]) };
    // Debug traces, do not remove yet
    //println!("accounts {:?}", accounts);
    //for account in accounts.iter() {
    //println!("account={:?}", account);
    //}

    let mut ret = vec![false; accounts.len()];

    // Deserialize the data into a tree
    let input_proof = unsafe { serialized_proof.to_vec() };
    let proof = rlp::decode::<Multiproof>(&input_proof).unwrap();
    let tree: Node = proof.rebuild()?;
    // Check that each account in present in the tree
    for (i, account) in accounts.iter().enumerate() {
        match account {
            Account::Empty => {}
            Account::Existing(addr, _, _, _) => {
                ret[i] = tree.has_key(addr);
            }
        }
    }
    Ok(ret)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let ok = verify().unwrap();
    unsafe {
        valid[..ok.len()].copy_from_slice(&ok[..]);
    }
}

#[cfg(test)]
mod tests {
    use super::multiproof_rs::node::*;
    use super::multiproof_rs::*;
    use super::*;

    #[test]
    fn test_recover_account_no_keys() {
        let mut root = Node::default();
        root.insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        root.insert(&NibbleKey::from(vec![1u8; 32]), vec![1u8; 32])
            .unwrap();
        let proof = make_multiproof(&root, vec![NibbleKey::from(vec![1u8; 32])]).unwrap();
        let encoding = rlp::encode(&proof);
        assert!(encoding.len() < unsafe { serialized_proof.len() });
        unsafe {
            &mut serialized_proof[..].copy_from_slice(&[0; 1024]);
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
        };

        // Verify that no answer is given since no question has been asked.
        assert_eq!(verify().unwrap().len(), 0);
    }

    #[test]
    fn test_validate_keys() {
        let mut root = Node::default();
        root.insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        root.insert(&NibbleKey::from(vec![1u8; 32]), vec![1u8; 32])
            .unwrap();
        let proof = make_multiproof(&root, vec![NibbleKey::from(vec![1u8; 32])]).unwrap();
        let encoding = rlp::encode(&proof);
        assert!(encoding.len() < unsafe { serialized_proof.len() });
        unsafe {
            &mut serialized_proof[..].copy_from_slice(&[0u8; 1024]);
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        let keys = rlp::encode_list(&vec![Account::Existing(
            NibbleKey::from(vec![1u8; 32]),
            0,
            vec![],
            false,
        )]);
        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
            &mut address_list[..keys.len()].copy_from_slice(&keys);
        };

        // Verify that the acount was indeed in the proof
        assert_eq!(verify().unwrap()[0], true);
    }

    #[test]
    fn test_catch_invalid_key() {
        let mut root = Node::default();
        root.insert(&NibbleKey::from(vec![0u8; 32]), vec![0u8; 32])
            .unwrap();
        root.insert(&NibbleKey::from(vec![1u8; 32]), vec![1u8; 32])
            .unwrap();
        let proof = make_multiproof(&root, vec![NibbleKey::from(vec![1u8; 32])]).unwrap();
        let encoding = rlp::encode(&proof);
        assert!(encoding.len() < unsafe { serialized_proof.len() });
        unsafe {
            &mut serialized_proof[..].copy_from_slice(&[0u8; 1024]);
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        let keys = rlp::encode_list(&vec![
            Account::Existing(NibbleKey::from(vec![15u8; 16]), 0, vec![], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![], false),
        ]);
        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
            &mut address_list[..keys.len()].copy_from_slice(&keys);
        };

        // Verify that the acount was indeed in the proof
        let result = verify();
        for res in result.unwrap().iter() {
            assert_eq!(*res, false);
        }
    }

    #[test]
    fn test_validate_keys_null() {
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
            &mut serialized_proof[..].copy_from_slice(&[0u8; 1024]);
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        let keys = rlp::encode_list::<Account, Account>(&vec![
            Account::Existing(NibbleKey::from(vec![1u8; 32]), 0, vec![], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![], false),
        ]);
        assert!(keys.len() < unsafe { address_list.len() });
        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
            &mut address_list[..keys.len()].copy_from_slice(&keys);
        };

        assert_eq!(verify().unwrap()[0], true);
        assert_eq!(verify().unwrap()[1], false);
    }

    #[test]
    fn code_decode_account() {
        let account = Account::Existing(NibbleKey::from(vec![1u8; 32]), 0, vec![10u8], false);
        let encoding = rlp::encode(&account);
        let decoded = rlp::decode::<Account>(&encoding).unwrap();
        assert_eq!(account, decoded);

        let accounts = vec![
            Account::Existing(NibbleKey::from(vec![1u8; 32]), 0, vec![10u8], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![10u8], false),
        ];
        let encoding = rlp::encode_list(&accounts);
        let decoded = rlp::decode_list::<Account>(&encoding);
        assert_eq!(accounts, decoded);
    }
}
