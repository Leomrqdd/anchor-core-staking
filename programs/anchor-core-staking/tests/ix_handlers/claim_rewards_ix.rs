use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System},
    anchor_spl::{associated_token::AssociatedToken, associated_token::get_associated_token_address, token::Token},
    solana_keypair::Keypair,
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn claim_rewards_ix(
    owner: &Keypair,
    config: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
    update_authority: Pubkey,
    rewards_mint: Pubkey,
) -> Instruction {
    let owner_rewards_ata = get_associated_token_address(&owner.pubkey(), &rewards_mint);

    Instruction::new_with_bytes(
        anchor_core_staking::id(),
        &anchor_core_staking::instruction::ClaimRewards {}.data(),
        anchor_core_staking::accounts::ClaimRewards {
            owner: owner.pubkey(),
            config,
            asset,
            collection,
            update_authority,
            rewards_mint,
            owner_rewards_ata,
            token_program: Token::id(),
            associated_token_program: AssociatedToken::id(),
            system_program: System::id(),
            mpl_core_program: mpl_core::ID,
        }
        .to_account_metas(None),
    )
}
