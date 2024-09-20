use crate::{APPROVE_ACCOUNT_SEED};
use anchor_lang::{
    prelude::Result,
    solana_program::{
        account_info::AccountInfo,
        program::invoke,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction::transfer,
        sysvar::Sysvar,
    },
    Lamports,
};

use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};

pub fn update_account_lamports_to_minimum_balance<'info>(
    account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let extra_lamports = Rent::get()?
        .minimum_balance(account.data_len())
        .saturating_sub(account.get_lamports());
    if extra_lamports > 0 {
        invoke(
            &transfer(payer.key, account.key, extra_lamports),
            &[payer, account, system_program],
        )?;
    }
    Ok(())
}

pub fn get_approve_account_pda(mint: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[APPROVE_ACCOUNT_SEED, mint.as_ref()], &crate::id()).0
}

pub fn get_meta_list(approve_account: Pubkey) -> Vec<ExtraAccountMeta> {
    vec![ExtraAccountMeta {
        discriminator: 0,
        address_config: approve_account.to_bytes(),
        is_signer: false.into(),
        is_writable: true.into(),
    }]
}

pub fn get_meta_list_size(approve_account: Pubkey) -> usize {
    ExtraAccountMetaList::size_of(get_meta_list(approve_account).len()).unwrap()
}
