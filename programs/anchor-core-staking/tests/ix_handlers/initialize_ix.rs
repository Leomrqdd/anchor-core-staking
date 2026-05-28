use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_instruction::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn initialize_ix(
    admin:&Keypair,
    config: Pubkey,
    collection: Pubkey,
    update_authority: Pubkey,
    rewards_mint: Pubkey,
) -> Instruction {
    
    Instruction::new_with_bytes(
        anchor_core_staking::id(),
        &anchor_core_staking::instruction::Initialize {
            rewards_bps: 500,
            freeze_period: 7,
        }
        .data(),
        anchor_core_staking::accounts::Initialize {
            admin: admin.pubkey(),
            config,
            collection,
            update_authority,
            rewards_mint,
            system_program: System::id(),
            token_program: Token::id(),

        }
        .to_account_metas(None),

    )
}

  