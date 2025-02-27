use solana_program::instruction::{Instruction, AccountMeta};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program_test::ProgramTest;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_sdk::account::Account;


/// 测试SOL批量转账功能
#[tokio::test]
async fn test_batch_transfer_sol() {
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

    // 执行批量转账
    let transfers = vec![
        (recipient1.pubkey(), 1 * LAMPORTS_PER_SOL),
        (recipient2.pubkey(), 2 * LAMPORTS_PER_SOL),
    ];

    let mut accounts = batch_transfer::accounts::BatchTransferSol {
        bank_account,
        sender: sender.pubkey(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    // 添加接收者账户
    accounts.extend(transfers.iter().map(|(pubkey, _)| AccountMeta::new(*pubkey, false)));

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

    banks_client.process_transaction(batch_transfer_tx).await.unwrap();

    // 验证转账结果
    let recipient1_balance = banks_client
        .get_account(recipient1.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let recipient2_balance = banks_client
        .get_account(recipient2.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let sender_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    assert_eq!(recipient1_balance, 1 * LAMPORTS_PER_SOL);
    assert_eq!(recipient2_balance, 2 * LAMPORTS_PER_SOL);
    assert!(sender_balance < 97 * LAMPORTS_PER_SOL); // 考虑手续费
    assert!(sender_balance > 96 * LAMPORTS_PER_SOL);
} 