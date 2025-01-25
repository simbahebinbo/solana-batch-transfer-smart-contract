use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use solana_sdk::instruction::Instruction;
use solana_sdk::account::Account;
use solana_sdk::system_program;
use spl_token::instruction as token_instruction;
use spl_token::solana_program::program_pack::Pack;

use batch_transfer;

#[tokio::test]
async fn test_check_balance_token() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let user = Keypair::new();

    // 为测试账户添加初始 SOL 余额
    let initial_sol_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: initial_sol_balance,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    pt.add_account(
        user.pubkey(),
        Account {
            lamports: initial_sol_balance,
            data: vec![],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // 创建代币相关账户
    let mint = Keypair::new();
    let user_token_account = Keypair::new();

    // 添加 mint 账户
    pt.add_account(
        mint.pubkey(),
        Account {
            lamports: 1000000000000,
            data: vec![0; spl_token::state::Mint::LEN],
            owner: spl_token::id().to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // 添加代币账户
    pt.add_account(
        user_token_account.pubkey(),
        Account {
            lamports: 10000000000000,
            data: vec![0; spl_token::state::Account::LEN],
            owner: spl_token::id().to_bytes().into(),
            executable: false,
            rent_epoch: 0,
        },
    );

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 初始化 mint
    let init_mint_ix = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey().to_bytes().into(),
        &payer.pubkey().to_bytes().into(),
        None,
        9,
    ).unwrap();

    let init_mint_tx = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id: spl_token::id().to_bytes().into(),
            accounts: init_mint_ix.accounts.iter().map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: meta.pubkey.to_bytes().into(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            }).collect(),
            data: init_mint_ix.data,
        }],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(init_mint_tx).await.unwrap();

    // 初始化代币账户
    let init_token_account_ix = token_instruction::initialize_account(
        &spl_token::id(),
        &user_token_account.pubkey().to_bytes().into(),
        &mint.pubkey().to_bytes().into(),
        &payer.pubkey().to_bytes().into(),
    ).unwrap();

    let init_token_account_tx = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id: spl_token::id().to_bytes().into(),
            accounts: init_token_account_ix.accounts.iter().map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: meta.pubkey.to_bytes().into(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            }).collect(),
            data: init_token_account_ix.data,
        }],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(init_token_account_tx).await.unwrap();

    // 铸造代币到用户账户
    let mint_to_ix = token_instruction::mint_to(
        &spl_token::id(),
        &mint.pubkey().to_bytes().into(),
        &user_token_account.pubkey().to_bytes().into(),
        &payer.pubkey().to_bytes().into(),
        &[&payer.pubkey().to_bytes().into()],
        1000000000,
    ).unwrap();

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[Instruction {
            program_id: spl_token::id().to_bytes().into(),
            accounts: mint_to_ix.accounts.iter().map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: meta.pubkey.to_bytes().into(),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            }).collect(),
            data: mint_to_ix.data,
        }],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(mint_to_tx).await.unwrap();

    // 测试检查代币余额
    let check_balance_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::CheckBalanceToken {
            token_account: user_token_account.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::CheckBalanceToken {}.data(),
    };

    let check_balance_tx = Transaction::new_signed_with_payer(
        &[check_balance_ix],
        Some(&user.pubkey()),
        &[&user],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(check_balance_tx).await;
    assert!(result.is_ok());

    // 验证代币余额
    let account = banks_client
        .get_account(user_token_account.pubkey())
        .await
        .unwrap()
        .unwrap();
    let token_account = spl_token::state::Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, 1000000000);
}

pub async fn load_and_deserialize<T: AccountDeserialize>(
    mut banks_client: BanksClient,
    address: Pubkey,
) -> T {
    let account = banks_client
        .get_account(address)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}
