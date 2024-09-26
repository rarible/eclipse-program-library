use anchor_lang::prelude::*;
use anchor_spl::token_interface::spl_token_2022::{
    extension::{BaseStateWithExtensions, StateWithExtensions},
    state::Mint as BaseStateMint,
};
use anchor_spl::token_interface::spl_token_metadata_interface::state::TokenMetadata;
use std::str::FromStr;
use crate::{PlatformFeeRecipient, UpdatePlatformFeeArgs, PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY};

pub fn get_platform_fee(account: &AccountInfo) -> Result<UpdatePlatformFeeArgs> {
    // Get the TokenMetadata from the mint account
    let mint_account_data = account.try_borrow_data()?;
    let mint_data = StateWithExtensions::<BaseStateMint>::unpack(&mint_account_data)?;
    let metadata = mint_data.get_variable_len_extension::<TokenMetadata>()?;

    // Initialize variables to store platform fee data
    let mut platform_fee_value: Option<u64> = None;
    let mut is_fee_flat: Option<bool> = None;
    let mut recipients_map: std::collections::BTreeMap<usize, PlatformFeeRecipient> =
        std::collections::BTreeMap::new();

    // Iterate over additional_metadata
    for (key, value) in metadata.additional_metadata.iter() {
        let key_str = key.as_str(); // Convert &String to &str for comparison

        if key_str == format!("{}is_fee_flat", PLATFORM_FEE_PREFIX_KEY) {
            // Parse value as bool
            let is_flat: bool = value.parse().map_err(|_| ProgramError::InvalidArgument)?;
            is_fee_flat = Some(is_flat);
        } else if key_str == format!("{}{}", PLATFORM_FEE_PREFIX_KEY, PLATFORM_FEE_VALUE_KEY) {
            // Parse value as u64
            let fee_value: u64 = value.parse().map_err(|_| ProgramError::InvalidArgument)?;
            platform_fee_value = Some(fee_value);
        } else if key_str.starts_with(PLATFORM_FEE_PREFIX_KEY) {
            // Key format: "platform_fee__recipients__<index>__<field>"
            let key_suffix = &key_str[PLATFORM_FEE_PREFIX_KEY.len()..]; // Remove the prefix
            let parts: Vec<&str> = key_suffix.split("__").collect();
            if parts.len() == 3 && parts[0] == "recipients" {
                let index: usize = parts[1].parse().map_err(|_| ProgramError::InvalidArgument)?;
                let field = parts[2];

                // Get or create the recipient at this index
                let recipient = recipients_map.entry(index).or_insert(PlatformFeeRecipient {
                    address: Pubkey::default(),
                    share: 0,
                });

                match field {
                    "address" => {
                        let pubkey = Pubkey::from_str(value).map_err(|_| ProgramError::InvalidArgument)?;
                        recipient.address = pubkey;
                    }
                    "share" => {
                        let share: u8 = value.parse().map_err(|_| ProgramError::InvalidArgument)?;
                        recipient.share = share;
                    }
                    _ => {
                        // Unrecognized field, ignore or handle error
                        return Err(ProgramError::InvalidArgument.into());
                    }
                }
            }
        }
    }

    // Ensure that platform_fee_value and is_fee_flat are set
    let fee_value = platform_fee_value.ok_or(ProgramError::InvalidArgument)?;
    let is_flat = is_fee_flat.ok_or(ProgramError::InvalidArgument)?;

    // Collect recipients into a vector, sorted by index
    let mut recipients = recipients_map
        .into_iter()
        .map(|(_, recipient)| recipient)
        .collect::<Vec<_>>();

    Ok(UpdatePlatformFeeArgs {
        platform_fee_value: fee_value,
        is_fee_flat: is_flat,
        recipients,
    })
}
