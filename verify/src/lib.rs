extern crate multiproof_rs;
extern crate rlp;

use multiproof_rs::{ByteKey, NibbleKey};

// The RLP-serialized proof
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut serialized_proof: &'static mut [u8] = &mut [0u8; 1024];

// An RLP-encoded list of accounts to be verified
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut address_list: &'static mut [u8] = &mut [0u8; 1024];

// An RLP-encoded list of accounts to be verified
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut address_list_size: usize = 0;

// An indicator whether the verification succeeded or not
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut valid: bool = false;

// Where the new root is to be stored
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut new_root: &'static mut [u8] = &mut [0u8; 256];

use multiproof_rs::{Multiproof, Node, ProofToTree};

// Address, value, code, state
#[derive(Debug, PartialEq)]
enum Account {
    Existing(NibbleKey, u32, Vec<u8>, bool),
    Empty,
}

impl rlp::Decodable for Account {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        match rlp.item_count()? {
            4 => {
                //let addr = rlp.val_at::<NibbleKey>(0)?;
                // XXX update multiproof to implement Into<Vec<u8>> for ByteKey so
                // that keys can be stored as bytes instead of nibbles, which would
                // make proofs shorter.
                let addr = NibbleKey::from(rlp.val_at::<Vec<u8>>(0)?);
                let balance = rlp.val_at(1)?;
                let code = rlp.val_at(2)?;
                let state = rlp.val_at(3)?;

                Ok(Account::Existing(addr, balance, code, state))
            }
            0 => Ok(Account::Empty),
            n => panic!(format!("Invalid payload {}", n)),
        }
    }
}

impl rlp::Encodable for Account {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        match self {
            Self::Empty => stream.append_empty_data(),
            Self::Existing(addr, balance, code, state) => {
                let mut s = rlp::RlpStream::new();
                s.begin_unbounded_list()
                    .append(addr)
                    .append(balance)
                    .append(code)
                    .append(state)
                    .finalize_unbounded_list();
                let encoding = s.out();
                stream.append_raw(&encoding, 1)
            }
        };
    }
}

fn verify() -> Result<bool, String> {
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

    // Deserialize the data into a tree
    let input_proof = unsafe { serialized_proof.to_vec() };
    let proof = rlp::decode::<Multiproof>(&input_proof).unwrap();
    match proof.rebuild() as Result<Node, String> {
        Ok(tree) => {
            // Check that each account in present in the tree
            for account in accounts.iter() {
                match account {
                    Account::Empty => {}
                    Account::Existing(addr, _, _, _) => {
                        if
                        /* !tree.has_key(addr) */
                        !tree.is_key_present(addr) {
                            return Err(format!("missing key {:?}", addr));
                        }
                    }
                }
            }

            unsafe { valid = true };
            Ok(true)
        }
        _ => Ok(false),
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() {
    let ok = verify().unwrap();
    unsafe {
        valid = ok;
    }
}

#[cfg(test)]
mod tests {
    use super::multiproof_rs::node::*;
    use super::multiproof_rs::utils::*;
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
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
        };

        // Verify that the acount was indeed in the proof
        assert_eq!(verify().unwrap(), true);
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
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        unsafe {
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
        };

        // Verify that the acount was indeed in the proof
        assert_eq!(verify().unwrap(), true);
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
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        let keys = rlp::encode_list(&vec![
            Account::Existing(NibbleKey::from(vec![15u8; 16]), 0, vec![], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![], false),
        ]);
        unsafe {
            address_list_size = keys.len() - 1;
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
            &mut address_list[..keys.len()].copy_from_slice(&keys);
        };

        // Verify that the acount was indeed in the proof
        let result = verify();
        assert!(result.is_err());
        result.expect_err("missing key");
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
            &mut serialized_proof[..encoding.len()].copy_from_slice(&encoding);
        };

        let keys = rlp::encode_list::<Account, Account>(&vec![
            Account::Existing(NibbleKey::from(vec![1u8; 32]), 0, vec![], false),
            Account::Existing(NibbleKey::from(vec![2u8; 32]), 0, vec![], false),
        ]);
        assert!(keys.len() < unsafe { address_list.len() });
        unsafe {
            address_list_size = keys.len() - 1;
            &mut address_list[..].copy_from_slice(&[0u8; 1024]);
            &mut address_list[..keys.len()].copy_from_slice(&keys);
        };

        // Verify that the acount was indeed in the proof
        assert_eq!(verify().unwrap(), true);
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
