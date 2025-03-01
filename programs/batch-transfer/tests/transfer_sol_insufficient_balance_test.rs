use solana_sdk::instruction::{Instruction, AccountMeta};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::account::Account;
use batch_transfer::TransferInfo;


/// 测试余额不足的情况
#[tokio::test]
async fn test_insufficient_balance() {
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();

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
            lamports: 1 * LAMPORTS_PER_SOL, // 只给发送者1个SOL
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
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::SetFee {
            bank_account,
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee {
            fee,
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

    // 尝试转账超过余额的金额
    let transfer_amount1 = LAMPORTS_PER_SOL * 2; // 2 SOL
    let transfer_amount2 = LAMPORTS_PER_SOL * 3; // 3 SOL

    let transfers = vec![
        TransferInfo {
            recipient: recipient1.pubkey(),
            amount: transfer_amount1,
        },
        TransferInfo {
            recipient: recipient2.pubkey(),
            amount: transfer_amount2,
        },
    ];

    let mut accounts = batch_transfer::accounts::BatchTransferSol {
        bank_account,
        sender: sender.pubkey(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    // 添加接收者账户
    accounts.extend(transfers.iter().map(|info| AccountMeta::new(info.recipient, false)));

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts,
        data: batch_transfer::instruction::BatchTransferSol {
            transfers: transfers.clone(),
        }
        .data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());

    // 验证余额未变（考虑交易费用）
    let sender_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert!(sender_balance > LAMPORTS_PER_SOL - 10000); // 允许最多 0.00001 SOL 的交易费用

    // 验证接收者未收到代币
    let recipient1_account = banks_client.get_account(recipient1.pubkey()).await.unwrap();
    assert!(recipient1_account.is_none());
    let recipient2_account = banks_client.get_account(recipient2.pubkey()).await.unwrap();
    assert!(recipient2_account.is_none());
} 