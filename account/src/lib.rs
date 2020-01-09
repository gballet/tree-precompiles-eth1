use multiproof_rs::NibbleKey;

// Address, value, code, state
#[derive(Debug, PartialEq)]
pub enum Account {
    Existing(NibbleKey, u32, Vec<u8>, bool),
    Empty,
}

impl rlp::Decodable for Account {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        match rlp.item_count()? {
            4 => {
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
            Account::Empty => {
                stream.append_empty_data();
            }
            Account::Existing(addr, balance, code, state) => {
                stream
                    .begin_unbounded_list()
                    .append(addr)
                    .append(balance)
                    .append(code)
                    .append(state)
                    .finalize_unbounded_list();
            }
        };
    }
}
