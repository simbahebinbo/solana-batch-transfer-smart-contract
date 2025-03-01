use solana_program::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::SysvarId,
};
use spl_token::state::Mint;
use spl_token::state::Account as TokenAccount;
use spl_token::{
    instruction as token_instruction,
    ID as TOKEN_PROGRAM_ID,
};
use solana_sdk::account::Account;
use batch_transfer::TransferInfo;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};
use std::str::FromStr;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_spl::token::{self};
use spl_token::solana_program::program_pack::Pack;

const PROGRAM_ID: &str = "De1JkAKuuvfrMhKKai7u53w8Ap8ufvF2QPigQMSMTyEh";

/// 将 Solana SDK 的 Pubkey 转换为 SPL Token 的 Pubkey
fn to_token_pubkey(pubkey: &Pubkey) -> spl_token::solana_program::pubkey::Pubkey {
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(pubkey.as_ref());
    spl_token::solana_program::pubkey::Pubkey::new_from_array(bytes)
}

/// 将 SPL Token 的 Instruction 转换为 Solana SDK 的 Instruction
fn to_sdk_instruction(ix: spl_token::solana_program::instruction::Instruction) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(ix.program_id.to_bytes()),
        accounts: ix.accounts.into_iter().map(|meta| AccountMeta {
            pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }).collect(),
        data: ix.data,
    }
}

/// 初始化代币铸币账户
async fn initialize_token_mint(
    banks_client: &mut solana_program_test::BanksClient,
    mint: &Keypair,
    authority: &Pubkey,
    decimals: u8,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);

    let space = Mint::LEN;
    let create_mint_account_ix = solana_sdk::system_instruction::create_account(
        authority,
        &mint.pubkey(),
        mint_rent,
        space as u64,
        &token::ID,
    );

    let initialize_mint_ix = token_instruction::initialize_mint(
        &to_token_pubkey(&token::ID),
        &to_token_pubkey(&mint.pubkey()),
        &to_token_pubkey(authority),
        Some(&to_token_pubkey(authority)),
        decimals,
    )
    .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, to_sdk_instruction(initialize_mint_ix)],
        Some(authority),
        &[mint],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

/// 创建代币账户
async fn create_token_account(
    banks_client: &mut solana_program_test::BanksClient,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let rent = banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(TokenAccount::LEN);

    let space = TokenAccount::LEN;
    let create_account_ix = solana_sdk::system_instruction::create_account(
        &owner.pubkey(),
        &account.pubkey(),
        account_rent,
        space as u64,
        &token::ID,
    );

    let initialize_account_ix = token_instruction::initialize_account(
        &to_token_pubkey(&token::ID),
        &to_token_pubkey(&account.pubkey()),
        &to_token_pubkey(mint),
        &to_token_pubkey(&owner.pubkey()),
    )
    .unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, to_sdk_instruction(initialize_account_ix)],
        Some(&owner.pubkey()),
        &[owner, account],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();
}

/// 测试空转账列表
#[test]
fn test_batch_transfer_token_empty() {
    solana_program_test::tokio::runtime::Runtime::new().unwrap().block_on(async {
        let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();
        let mut pt = ProgramTest::new("batch_transfer", program_id, None);
        pt.set_compute_max_units(1200_000);

        let admin = Keypair::new();
        let sender = Keypair::new();
        let mint = Keypair::new();
        let sender_token = Keypair::new();

        // 计算bank_account的PDA
        let (bank_account, _bump) = Pubkey::find_program_address(
            &[b"bank_account"],
            &program_id,
        );

        // 为管理员和发送者添加初始余额
        pt.add_account(
            admin.pubkey(),
            Account {
                lamports: 100 * LAMPORTS_PER_SOL,
                ..Account::default()
            },
        );
        pt.add_account(
            sender.pubkey(),
            Account {
                lamports: 100 * LAMPORTS_PER_SOL,
                ..Account::default()
            },
        );

        let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

        // 初始化银行账户
        let initialize_ix = Instruction {
            program_id: batch_transfer::ID,
            accounts: batch_transfer::accounts::Initialize {
                bank_account,
                deployer: admin.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: batch_transfer::instruction::Initialize {
                admin: admin.pubkey(),
            }
            .data(),
        };

        let initialize_tx = Transaction::new_signed_with_payer(
            &[initialize_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(initialize_tx).await.unwrap();

        // 设置手续费
        let set_fee_ix = Instruction {
            program_id: batch_transfer::ID,
            accounts: batch_transfer::accounts::SetFee {
                bank_account,
                admin: admin.pubkey(),
            }
            .to_account_metas(None),
            data: batch_transfer::instruction::SetFee {
                fee: 1000,
            }
            .data(),
        };

        let set_fee_tx = Transaction::new_signed_with_payer(
            &[set_fee_ix],
            Some(&admin.pubkey()),
            &[&admin],
            recent_blockhash,
        );

        banks_client.process_transaction(set_fee_tx).await.unwrap();

        // 初始化代币
        initialize_token_mint(&mut banks_client, &mint, &admin.pubkey(), 6, recent_blockhash).await;
        create_token_account(&mut banks_client, &sender_token, &mint.pubkey(), &sender, recent_blockhash).await;

        // 执行空转账列表
        let transfers: Vec<TransferInfo> = vec![];

        let accounts = batch_transfer::accounts::BatchTransferToken {
            bank_account,
            sender: sender.pubkey(),
            token_account: sender_token.pubkey(),
            token_program: token::ID,
            system_program: system_program::ID,
        }.to_account_metas(None);

        let batch_transfer_ix = Instruction {
            program_id: batch_transfer::ID,
            accounts,
            data: batch_transfer::instruction::BatchTransferToken {
                transfers,
            }.data(),
        };

        let batch_transfer_tx = Transaction::new_signed_with_payer(
            &[batch_transfer_ix],
            Some(&sender.pubkey()),
            &[&sender],  // 只需要 sender 作为签名者，因为它是 token_account 的所有者
            recent_blockhash,
        );

        let result = banks_client.process_transaction(batch_transfer_tx).await;
        assert!(result.is_err()); // 空转账列表应该返回错误
    });
} 