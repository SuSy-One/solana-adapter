use std::fmt;
use std::error;

use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub type WrappedResult<T> = Result<T, Box<dyn error::Error>>;


#[derive(PartialEq, PartialOrd, Default, Clone)]
pub struct GravityContract {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,

    pub bft: u8,
    pub consuls: Vec<Pubkey>,
    pub last_round: u64
}

impl fmt::Display for GravityContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let consuls_joined: Vec<String> = self.consuls.iter().map(|x| { &x.to_bytes().unwrap() }).collect();
        write!(
            f,
            "is_initialized: {:}; initializer_pubkey: {:}; bft: {:}; last_round: {:}",
            self.is_initialized, self.initializer_pubkey, self.bft, self.last_round
        )
    }
}

impl Sealed for GravityContract {}

impl IsInitialized for GravityContract {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for GravityContract {
    const LEN: usize = 138;

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, GravityContract::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            bft,
            consuls,
            last_round,
        ) = array_refs![src, 1, 32, 1, 32 * 3, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(GravityContract {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            bft: u8::from_le_bytes(*bft),
            consuls: vec![
                Pubkey::new_from_array(*array_ref![consuls[0..32], 0, 32]),
                Pubkey::new_from_array(*array_ref![consuls[32..64], 0, 32]),
                Pubkey::new_from_array(*array_ref![consuls[64..96], 0, 32]),
            ],
            last_round: u64::from_le_bytes(*last_round),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, GravityContract::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            bft_dst,
            consuls_dst,
            last_round_dst,
        ) = mut_array_refs![dst, 1, 32, 1, 32 * 3, 8];

        let GravityContract {
            is_initialized,
            initializer_pubkey,
            bft,
            consuls,
            last_round,
        } = self;
        
        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        bft_dst[0] = *bft as u8;
        
        let consuls_copy = consuls.clone();
        consuls_dst.copy_from_slice(
            consuls_copy
                .iter()
                .fold(vec![], |acc,x| { vec![acc, x.to_bytes().to_vec()].concat() })
                .as_slice()
        );

        *last_round_dst = last_round.to_le_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate hex;
    extern crate rand;
    
    use rand::random;
        

    fn build_gravity_contract_mock() -> GravityContract {
        let mock_gravity_consuls = vec![
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mock_bft: u8 = random();
        let mock_last_round: u64 = random();

        let gravity_contract_mock = GravityContract {
            consuls: mock_gravity_consuls.clone(),
            bft: mock_bft,
            last_round: mock_last_round,
            ..GravityContract::default()
        };

        gravity_contract_mock
    }

    // test serialize and deserialize to prove internal algo is correct
    #[test]
    fn test_ser_deser_internal() -> WrappedResult<()> {
        let gravity_contract_mock = build_gravity_contract_mock();

        // serialize
        let mut serialized_gravity_contract_bytes = [0 as u8; GravityContract::LEN];

        // populate byte slice
        gravity_contract_mock.pack_into_slice(&mut serialized_gravity_contract_bytes);

        // deserialize
        let deserialized_gravity_contract = GravityContract::unpack_from_slice(&mut serialized_gravity_contract_bytes)
            .expect("deserialization failed!");

        assert!(deserialized_gravity_contract == gravity_contract_mock);

        Ok(())
    }

    // test serialize and deserialize using raw methods
    #[test]
    fn test_raw_tx_deser() -> WrappedResult<()> {


        let raw_tx_inputs = vec![
            "01130552cdea768b3a63553a978383d007e6e1c4be5c3544cd2a657c31720aef51a2a5e31a12722fdbe3e7ac8877467fa0389487c5a4725795506ff8dbcd85910301000103bfb92919a3a0f16abc73951e82c05592732e5514ffa5cdae5f77a96d04922c853b243370dff1af837da92b91fc34b6b25bc35c011fdc1061512a3a01ea324b06be8f3dc36da246f1c085fd38b1591451bde88f5681ad8418bc6098ae2852d8da866463c16e94fc8fa3345d678c24a0703f3dfa24d49af313b4279d7e6d8ee5ed01020200016100cf0a594a522816ef0953a69843607a51450c928f3c23ba552c1a6262ac43430787fd12467b9ad4cff20aaa8b5b8850c29165d68d5d17eb571f143f72842a12ab7e143ebaf52b647ce4c4d1fb57ba3e1d3a6da3ff9300feff288c389146e54bd9"
        ];
        
        for (i, input) in raw_tx_inputs.iter().enumerate() {
            // let decoded_string = hex::decode("48656c6c6f20776f726c6421");
            let mut serialized_gravity_contract_bytes = hex::decode(input)
            .expect("hex string to bytes cast failed!");

            // deserialize
            let deserialized_gravity_contract = GravityContract::unpack_from_slice(&mut serialized_gravity_contract_bytes)
                .expect("deserialization failed!");

            println!("contract #{:} from raw tx: \n {:} \n", i, deserialized_gravity_contract);
        }

        Ok(())
    }
}