use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use solana_program::instruction::Instruction;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;

use batch_transfer;

const LAMPORTS_PER_SOL: u64 = 1_000_000_000; // 1 SOL = 1_000_000_000 lamports
const ACCOUNT_RENT_EXEMPT_LAMPORTS: u64 = 890_880; // 最小租金豁免金额

#[tokio::test]
async fn test_check_balance_sol() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let account_with_balance = Keypair::new();
    let account_min_balance = Keypair::new();

    // 为第一个账户添加初始余额
    let initial_balance = LAMPORTS_PER_SOL; // 1 SOL
    pt.add_account(
        account_with_balance.pubkey(),
        Account {
            lamports: initial_balance,
            ..Account::default()
        },
    );

    // 为第二个账户添加最小租金余额
    pt.add_account(
        account_min_balance.pubkey(),
        Account {
            lamports: ACCOUNT_RENT_EXEMPT_LAMPORTS,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    // 测试检查有余额的账户
    let check_balance_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::CheckBalanceSol {
            account: account_with_balance.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::CheckBalanceSol {}.data(),
    };

    let check_balance_tx = Transaction::new_signed_with_payer(
        &[check_balance_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(check_balance_tx).await;
    assert!(result.is_ok());

    // 验证账户余额
    let account_data = banks_client
        .get_account(account_with_balance.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account_data.lamports, initial_balance);

    // 测试检查最小余额账户
    let check_min_balance_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::CheckBalanceSol {
            account: account_min_balance.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::CheckBalanceSol {}.data(),
    };

    let check_min_balance_tx = Transaction::new_signed_with_payer(
        &[check_min_balance_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(check_min_balance_tx).await;
    assert!(result.is_ok());

    // 验证最小余额账户
    let account_data = banks_client
        .get_account(account_min_balance.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(account_data.lamports, ACCOUNT_RENT_EXEMPT_LAMPORTS);
}
