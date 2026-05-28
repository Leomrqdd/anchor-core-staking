use {
    anchor_spl::associated_token::{self, get_associated_token_address},
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    litesvm_token::MintTo,
    litesvm_token::Transfer,
    litesvm_token::CreateAssociatedTokenAccount,
    solana_instruction::Instruction,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    solana_pubkey::Pubkey,
    anchor_core_staking::state::Config,
    anchor_lang::AccountDeserialize,
    anchor_spl::token::TokenAccount,
    mpl_core::{
        accounts::{BaseCollectionV1,BaseAssetV1},
        fetch_plugin,
        types::{Attributes, FreezeDelegate, PluginType},
    },
    solana_account_info::AccountInfo,
};

use solana_clock::Clock;

mod ix_handlers;
use ix_handlers::*;



fn send(
    svm: &mut LiteSVM,
    ixs:&[Instruction],
    payer: &Keypair,
    signers: &[&dyn Signer]
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn setup() -> (
    LiteSVM,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = anchor_core_staking::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_core_staking.so");
    let mpl_core_bytes = include_bytes!("fixtures/mpl_core.so");
    svm.add_program(mpl_core::ID, mpl_core_bytes);
    svm.add_program(program_id, bytes);
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();


    let collection = Keypair::new();

    let config = Pubkey::find_program_address(
        &[b"config", collection.pubkey().as_ref()],
        &program_id,
    ).0;

    let update_authority = Pubkey::find_program_address(
        &[b"update_authority", collection.pubkey().as_ref()],
        &program_id,
    ).0;

    let rewards_mint = Pubkey::find_program_address(
        &[b"reward_mint", config.as_ref()],
        &program_id,
    ).0;

    (
        svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    )

}



#[test]

fn test_create_collection() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let ix = create_collection_ix(
        &payer,
        &collection,
        update_authority,
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer, &collection]);
    assert!(res.is_ok());

    let account  = svm.get_account(&collection.pubkey()).unwrap();
    let collection_data = BaseCollectionV1::from_bytes(&account.data).unwrap();
    assert_eq!(collection_data.name, "Test Collection".to_string());
    assert_eq!(collection_data.uri, "https://example.com/collection.json".to_string());
    assert_eq!(collection_data.update_authority, update_authority);

}



#[test]
fn test_initialize() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let create_ix = create_collection_ix(&payer, &collection, update_authority);
    send(&mut svm, &[create_ix], &payer, &[&payer, &collection]).unwrap();

    let ix = initialize_ix(
        &payer,
        config,
        collection.pubkey(),
        update_authority,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_data = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_data.rewards_bps, 500);
    assert_eq!(config_data.freeze_period, 7);
    
}




#[test]
fn test_mint_asset() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let create_ix = create_collection_ix(&payer, &collection, update_authority);
    send(&mut svm, &[create_ix], &payer, &[&payer, &collection]).unwrap();

    let ix = initialize_ix(
        &payer,
        config,
        collection.pubkey(),
        update_authority,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_data = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_data.rewards_bps, 500);
    assert_eq!(config_data.freeze_period, 7);


    let asset = Keypair::new();
    let mint_ix = mint_asset_ix(&payer,&asset, collection.pubkey(), update_authority);
    let res = send(&mut svm, &[mint_ix], &payer, &[&payer, &asset]);
    assert!(res.is_ok(), "{:?}", res.err());

    let account  = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.name, "Test Asset".to_string());
    assert_eq!(asset_data.owner, payer.pubkey());

}




#[test]
fn test_stake() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let create_ix = create_collection_ix(&payer, &collection, update_authority);
    send(&mut svm, &[create_ix], &payer, &[&payer, &collection]).unwrap();

    let ix = initialize_ix(
        &payer,
        config,
        collection.pubkey(),
        update_authority,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_data = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_data.rewards_bps, 500);
    assert_eq!(config_data.freeze_period, 7);


    let asset = Keypair::new();
    let mint_ix = mint_asset_ix(&payer,&asset, collection.pubkey(), update_authority);
    let res = send(&mut svm, &[mint_ix], &payer, &[&payer, &asset]);
    assert!(res.is_ok());

    let account  = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.name, "Test Asset".to_string());
    assert_eq!(asset_data.owner, payer.pubkey());



    let ix = stake_ix(
        &payer,
        config,
        asset.pubkey(),
        collection.pubkey(),
        update_authority
    );
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let mut lamports = account.lamports;
    let mut data = account.data.clone();
    let mut asset_pubkey = asset.pubkey();
    let mut account_info = AccountInfo::new(
        &asset_pubkey,
        false,
        false,
        &mut lamports,
        &mut data,
        &account.owner,
        account.executable,
        account.rent_epoch,
    );

    let (_, freeze_delegate, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &account_info, PluginType::FreezeDelegate
    ).unwrap();
    assert!(freeze_delegate.frozen);


    let (_, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
        &account_info, PluginType::Attributes
    ).unwrap();
    let staked = attributes.attribute_list.iter().find(|a| a.key == "staked").unwrap();
    assert_eq!(staked.value, "true");
     



}



#[test]
fn test_unstake() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let create_ix = create_collection_ix(&payer, &collection, update_authority);
    send(&mut svm, &[create_ix], &payer, &[&payer, &collection]).unwrap();

    let ix = initialize_ix(
        &payer,
        config,
        collection.pubkey(),
        update_authority,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_data = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_data.rewards_bps, 500);
    assert_eq!(config_data.freeze_period, 7);


    let asset = Keypair::new();
    let mint_ix = mint_asset_ix(&payer,&asset, collection.pubkey(), update_authority);
    let res = send(&mut svm, &[mint_ix], &payer, &[&payer, &asset]);
    assert!(res.is_ok());

    let account  = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.name, "Test Asset".to_string());
    assert_eq!(asset_data.owner, payer.pubkey());



    let ix = stake_ix(
        &payer,
        config,
        asset.pubkey(),
        collection.pubkey(),
        update_authority
    );
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let mut lamports = account.lamports;
    let mut data = account.data.clone();
    let mut asset_pubkey = asset.pubkey();
    let mut account_info = AccountInfo::new(
        &asset_pubkey,
        false,
        false,
        &mut lamports,
        &mut data,
        &account.owner,
        account.executable,
        account.rent_epoch,
    );

    let (_, freeze_delegate, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &account_info, PluginType::FreezeDelegate
    ).unwrap();
    assert!(freeze_delegate.frozen);


    let (_, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
        &account_info, PluginType::Attributes
    ).unwrap();
    let staked = attributes.attribute_list.iter().find(|a| a.key == "staked").unwrap();
    assert_eq!(staked.value, "true");



    let mut clock: Clock = svm.get_sysvar();
    const EIGHT_DAYS_SECS: i64 = 691_200; // 8 * 24 * 60 * 60
    clock.unix_timestamp = clock.unix_timestamp.checked_add(EIGHT_DAYS_SECS).unwrap();
    svm.set_sysvar(&clock);



    let ix = unstake_ix(
        &payer,
        config,
        asset.pubkey(),
        collection.pubkey(),
        update_authority
    );
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:?}", res.err());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let mut lamports = account.lamports;
    let mut data = account.data.clone();
    let mut asset_pubkey = asset.pubkey();
    let mut account_info = AccountInfo::new(
        &asset_pubkey,
        false,
        false,
        &mut lamports,
        &mut data,
        &account.owner,
        account.executable,
        account.rent_epoch,
    );




    let (_, freeze_delegate, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &account_info, PluginType::FreezeDelegate
    ).unwrap();
    assert!(freeze_delegate.frozen == false);


    let (_, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
        &account_info, PluginType::Attributes
    ).unwrap();
    let staked = attributes.attribute_list.iter().find(|a| a.key == "staked").unwrap();
    assert_eq!(staked.value, "false");
}





#[test]
fn test_claim_rewards() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint,
    ) = setup();

    send(&mut svm, &[create_collection_ix(&payer, &collection, update_authority)], &payer, &[&payer, &collection]).unwrap();
    send(&mut svm, &[initialize_ix(&payer, config, collection.pubkey(), update_authority, rewards_mint)], &payer, &[&payer]).unwrap();

    let asset = Keypair::new();
    send(&mut svm, &[mint_asset_ix(&payer, &asset, collection.pubkey(), update_authority)], &payer, &[&payer, &asset]).unwrap();
    send(&mut svm, &[stake_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority)], &payer, &[&payer]).unwrap();

    const EIGHT_DAYS_SECS: i64 = 691_200;
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = clock.unix_timestamp.checked_add(EIGHT_DAYS_SECS).unwrap();
    svm.set_sysvar(&clock);

    let ix = claim_rewards_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority, rewards_mint);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:?}", res);

    // 8 days * 500 bps * 10^6 decimals / 10000 = 400_000
    let ata = get_associated_token_address(&payer.pubkey(), &rewards_mint);
    let ata_account = svm.get_account(&ata).unwrap();
    let token_data = TokenAccount::try_deserialize(&mut ata_account.data.as_slice()).unwrap();
    assert_eq!(token_data.amount, 400_000);
    assert_eq!(token_data.mint, rewards_mint);
    assert_eq!(token_data.owner, payer.pubkey());
}


#[test]
fn test_claim_rewards_without_staking() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint,
    ) = setup();

    send(&mut svm, &[create_collection_ix(&payer, &collection, update_authority)], &payer, &[&payer, &collection]).unwrap();
    send(&mut svm, &[initialize_ix(&payer, config, collection.pubkey(), update_authority, rewards_mint)], &payer, &[&payer]).unwrap();

    let asset = Keypair::new();
    send(&mut svm, &[mint_asset_ix(&payer, &asset, collection.pubkey(), update_authority)], &payer, &[&payer, &asset]).unwrap();

    const EIGHT_DAYS_SECS: i64 = 691_200;
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = clock.unix_timestamp.checked_add(EIGHT_DAYS_SECS).unwrap();
    svm.set_sysvar(&clock);

    let ix = claim_rewards_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority, rewards_mint);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_err());
}



#[test]
fn test_unstake_before_frozen_period() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint
    ) = setup();

    let create_ix = create_collection_ix(&payer, &collection, update_authority);
    send(&mut svm, &[create_ix], &payer, &[&payer, &collection]).unwrap();

    let ix = initialize_ix(
        &payer,
        config,
        collection.pubkey(),
        update_authority,
        rewards_mint
    );

    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&config).unwrap();
    let config_data = Config::try_deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(config_data.rewards_bps, 500);
    assert_eq!(config_data.freeze_period, 7);


    let asset = Keypair::new();
    let mint_ix = mint_asset_ix(&payer,&asset, collection.pubkey(), update_authority);
    let res = send(&mut svm, &[mint_ix], &payer, &[&payer, &asset]);
    assert!(res.is_ok());

    let account  = svm.get_account(&asset.pubkey()).unwrap();
    let asset_data = BaseAssetV1::from_bytes(&account.data).unwrap();
    assert_eq!(asset_data.name, "Test Asset".to_string());
    assert_eq!(asset_data.owner, payer.pubkey());



    let ix = stake_ix(
        &payer,
        config,
        asset.pubkey(),
        collection.pubkey(),
        update_authority
    );
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok());

    let account = svm.get_account(&asset.pubkey()).unwrap();
    let mut lamports = account.lamports;
    let mut data = account.data.clone();
    let mut asset_pubkey = asset.pubkey();
    let mut account_info = AccountInfo::new(
        &asset_pubkey,
        false,
        false,
        &mut lamports,
        &mut data,
        &account.owner,
        account.executable,
        account.rent_epoch,
    );

    let (_, freeze_delegate, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
        &account_info, PluginType::FreezeDelegate
    ).unwrap();
    assert!(freeze_delegate.frozen);


    let (_, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
        &account_info, PluginType::Attributes
    ).unwrap();
    let staked = attributes.attribute_list.iter().find(|a| a.key == "staked").unwrap();
    assert_eq!(staked.value, "true");

    // NO CLOCK

    let ix = unstake_ix(
        &payer,
        config,
        asset.pubkey(),
        collection.pubkey(),
        update_authority
    );
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_err());
}



#[test]
fn test_claim_rewards_reset_timer() {
    let (
        mut svm,
        payer,
        collection,
        config,
        update_authority,
        rewards_mint,
    ) = setup();

    send(&mut svm, &[create_collection_ix(&payer, &collection, update_authority)], &payer, &[&payer, &collection]).unwrap();
    send(&mut svm, &[initialize_ix(&payer, config, collection.pubkey(), update_authority, rewards_mint)], &payer, &[&payer]).unwrap();

    let asset = Keypair::new();
    send(&mut svm, &[mint_asset_ix(&payer, &asset, collection.pubkey(), update_authority)], &payer, &[&payer, &asset]).unwrap();
    send(&mut svm, &[stake_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority)], &payer, &[&payer]).unwrap();

    const EIGHT_DAYS_SECS: i64 = 691_200;
    let mut clock: Clock = svm.get_sysvar();
    clock.unix_timestamp = clock.unix_timestamp.checked_add(EIGHT_DAYS_SECS).unwrap();
    svm.set_sysvar(&clock);

    let ix = claim_rewards_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority, rewards_mint);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:?}", res);

    // 8 days * 500 bps * 10^6 decimals / 10000 = 400_000
    let ata = get_associated_token_address(&payer.pubkey(), &rewards_mint);
    let ata_account = svm.get_account(&ata).unwrap();
    let token_data = TokenAccount::try_deserialize(&mut ata_account.data.as_slice()).unwrap();
    assert_eq!(token_data.amount, 400_000);
    assert_eq!(token_data.mint, rewards_mint);
    assert_eq!(token_data.owner, payer.pubkey());


    // CLAIM AGAIN 
    let ix = claim_rewards_ix(&payer, config, asset.pubkey(), collection.pubkey(), update_authority, rewards_mint);
    let res = send(&mut svm, &[ix], &payer, &[&payer]);
    assert!(res.is_ok(), "{:?}", res);

    // NO MORE TOKENS
    let ata = get_associated_token_address(&payer.pubkey(), &rewards_mint);
    let ata_account = svm.get_account(&ata).unwrap();
    let token_data = TokenAccount::try_deserialize(&mut ata_account.data.as_slice()).unwrap();
    assert_eq!(token_data.amount, 400_000);
    assert_eq!(token_data.mint, rewards_mint);
    assert_eq!(token_data.owner, payer.pubkey());




}
