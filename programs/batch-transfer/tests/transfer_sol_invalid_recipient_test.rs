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


/// 测试无效接收者的情况
#[tokio::test]
async fn test_invalid_recipient() {
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let invalid_recipient = Pubkey::new_unique(); // 创建一个不存在的账户地址

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
    pt.add_account(
        recipient1.pubkey(),
        Account {
            lamports: 0,
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

    // 尝试转账到无效账户
    let transfers = vec![
        TransferInfo {
            recipient: recipient1.pubkey(),
            amount: 1 * LAMPORTS_PER_SOL,
        },
        TransferInfo {
            recipient: invalid_recipient,
            amount: 2 * LAMPORTS_PER_SOL,
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

    // 转账应该成功
    banks_client.process_transaction(batch_transfer_tx).await.unwrap();

    // 验证余额变化
    let sender_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(sender_balance, 96989995000);

    // 验证有效接收者收到代币
    let recipient1_balance = banks_client
        .get_account(recipient1.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient1_balance, 1 * LAMPORTS_PER_SOL);

    // 验证无效接收者也收到代币
    let invalid_recipient_balance = banks_client
        .get_account(invalid_recipient)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(invalid_recipient_balance, 2 * LAMPORTS_PER_SOL);
} 