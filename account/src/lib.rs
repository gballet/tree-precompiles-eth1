use multiproof_rs::NibbleKey;

#[derive(Debug, PartialEq)]
pub enum Account {
    // Address, nonce, value, code, state
    Existing(NibbleKey, u64, u64, Vec<u8>, bool),
    Empty,
}

impl rlp::Decodable for Account {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        match rlp.item_count()? {
            5 => {
                // XXX update multiproof to implement Into<Vec<u8>> for ByteKey so
                // that keys can be stored as bytes instead of nibbles, which would
                // make proofs shorter.
                let addr = NibbleKey::from(rlp.val_at::<Vec<u8>>(0)?);
                let nonce = rlp.val_at(1)?;
                let balance = rlp.val_at(2)?;
                let code = rlp.val_at(3)?;
                let state = rlp.val_at(4)?;

                Ok(Account::Existing(addr, nonce, balance, code, state))
            }
            0 => Ok(Account::Empty),
            n => panic!(format!("Invalid payload, item count={}", n)),
        }
    }
}

impl rlp::Encodable for Account {
    fn rlp_append(&self, stream: &mut rlp::RlpStream) {
        match self {
            Account::Empty => {
                stream.append_empty_data();
            }
            Account::Existing(addr, nonce, balance, code, state) => {
                stream
                    .begin_unbounded_list()
                    .append(addr)
                    .append(nonce)
                    .append(balance)
                    .append(code)
                    .append(state)
                    .finalize_unbounded_list();
            }
        };
    }
}
