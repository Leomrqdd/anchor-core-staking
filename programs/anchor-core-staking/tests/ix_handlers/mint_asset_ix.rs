use {
    anchor_lang::{Id, InstructionData, ToAccountMetas, prelude::System}, anchor_spl::{associated_token::AssociatedToken, mint, token::Token}, solana_keypair::Keypair, solana_instruction::Instruction, solana_pubkey::Pubkey, solana_signer::Signer
};

pub fn mint_asset_ix(
    user:&Keypair,
    asset: &Keypair,
    collection: Pubkey,
    update_authority: Pubkey,
) -> Instruction {
    
    Instruction::new_with_bytes(
        anchor_core_staking::id(),
        &anchor_core_staking::instruction::MintAsset {
            name: "Test Asset".to_string(),
            uri: "https://example.com/asset.json".to_string(),  
        }
        .data(),
        anchor_core_staking::accounts::MintAsset {
            user: user.pubkey(),
            asset: asset.pubkey(),
            collection,
            update_authority,
            system_program: System::id(),
            mpl_core_program: mpl_core::ID,
            
        }
        .to_account_metas(None),

    )
}

  