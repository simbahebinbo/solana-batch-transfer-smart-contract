use std::rc::Rc;
use std::sync::Arc;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::Client;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;

#[tokio::test]
async fn test_initialize() {
    let SetUpTest {
        program_id,
        deployer,
        admin,
        bank_account,
        // mut client,
    } = SetUpTest::new().await;

    // // 构建 Initialize 指令
    // let initialize_ix = Instruction {
    //     program_id,
    //     accounts: batch_transfer::accounts::Initialize {
    //         bank_account: bank_account.pubkey(),
    //         deployer: deployer.pubkey(),
    //         system_program: system_program::ID,
    //     }
    //         .to_account_metas(None),
    //     data: batch_transfer::instruction::Initialize {
    //         admin: admin.pubkey(),
    //     }
    //         .data(),
    // };
    //
    // // 构建并发送交易
    // let recent_blockhash = client.get_latest_blockhash().await.unwrap();
    // let initialize_tx = Transaction::new_signed_with_payer(
    //     &[initialize_ix],
    //     Some(&deployer.pubkey()),
    //     &[&deployer, &bank_account],
    //     recent_blockhash,
    // );
    //
    // client.send_transaction(&initialize_tx).await.unwrap();
    //
    // // 检查 bank_account 是否正确设置了 admin
    // let bank_account_data: batch_transfer::StakingAccount = client
    //     .get_account_data::<batch_transfer::StakingAccount>(bank_account.pubkey())
    //     .await
    //     .unwrap();
    //
    // assert_eq!(bank_account_data.admin, admin.pubkey());
}

pub struct SetUpTest {
    pub program_id: Pubkey,
    pub deployer: Keypair,
    pub admin: Keypair,
    pub bank_account: Keypair,
    // pub client: Client<Pubkey>,
}

impl SetUpTest {
    pub async fn new() -> Self {
        let program_id = batch_transfer::ID;

        // let wallet = Keypair::new();
        // let client = Client::new_with_options(
        //     anchor_client::Cluster::Localnet,
        //     wallet,
        //     CommitmentConfig::processed(),
        // );

        // 初始化账户
        let deployer = Keypair::new();
        let admin = Keypair::new();
        let bank_account = Keypair::new();

        // // 在链上创建并初始化这些账户
        // client
        //     .airdrop(&deployer.pubkey(), 1_000_000_000)
        //     .await
        //     .unwrap();
        // client
        //     .airdrop(&admin.pubkey(), 1_000_000_000)
        //     .await
        //     .unwrap();
        // client
        //     .airdrop(&bank_account.pubkey(), 1_000_000_000)
        //     .await
        //     .unwrap();

        Self {
            program_id,
            deployer,
            admin,
            bank_account,
            // client,
        }
    }
}
