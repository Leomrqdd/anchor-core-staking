use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_instruction::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn unstake_ix(
    owner:&Keypair,
    config: Pubkey,
    asset: Pubkey,
    collection: Pubkey,
    update_authority: Pubkey,
) -> Instruction {
    
    Instruction::new_with_bytes(
        anchor_core_staking::id(),
        &anchor_core_staking::instruction::Unstake {
        }
        .data(),
        anchor_core_staking::accounts::Unstake {
            owner: owner.pubkey(),
            config,
            asset,  
            collection,
            update_authority,
            system_program: System::id(),
            mpl_core_program: mpl_core::ID,
            
        }
        .to_account_metas(None),

    )
}

  