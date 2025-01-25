use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::system_program;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

use batch_transfer::BankAccount;

const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

/// 辅助函数：初始化测试环境
async fn setup_test_env() -> (ProgramTest, Keypair, Keypair, Keypair) {
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    let admin = Keypair::new();
    let sender = Keypair::new();
    let bank_account = Keypair::new();

    // 为管理员和发送者添加初始余额
    let initial_balance = 100 * LAMPORTS_PER_SOL;
    for account in [&admin, &sender] {
        pt.add_account(
            account.pubkey(),
            Account {
                lamports: initial_balance,
                ..Account::default()
            },
        );
    }

    (pt, admin, sender, bank_account)
}

/// 辅助函数：初始化 bank_account
async fn initialize_bank_account(
    banks_client: &mut solana_program_test::BanksClient,
    admin: &Keypair,
    bank_account: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let program_id = batch_transfer::ID;
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
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
        &[admin, bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();
}

/// 辅助函数：设置手续费
async fn set_fee(
    banks_client: &mut solana_program_test::BanksClient,
    admin: &Keypair,
    bank_account: &Keypair,
    fee: u64,
    recent_blockhash: solana_sdk::hash::Hash,
) {
    let program_id = batch_transfer::ID;
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();
}

#[tokio::test]
async fn test_batch_transfer_sol() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额
    let initial_sender_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sender_balance,
            ..Account::default()
        },
    );

    // 为管理员账户添加余额
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000_000, // 1 SOL
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
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
        &[&admin, &bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费
    let fee = 10_000_000; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();

    // 准备批量转账数据
    let transfer_amount1 = 100_000_000; // 0.1 SOL
    let transfer_amount2 = 200_000_000; // 0.2 SOL
    let transfers = vec![
        (recipient1.pubkey(), transfer_amount1),
        (recipient2.pubkey(), transfer_amount2),
    ];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client
        .process_transaction(batch_transfer_tx)
        .await
        .unwrap();

    // 验证转账结果
    let total_transfer = transfer_amount1 + transfer_amount2;
    let transaction_fee = 5000; // 预估的交易费用
    let expected_sender_balance = initial_sender_balance - total_transfer - fee - transaction_fee;

    // 检查发送者余额
    let sender_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(sender_balance, expected_sender_balance);

    // 检查接收者余额
    let recipient1_balance = banks_client
        .get_account(recipient1.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient1_balance, transfer_amount1);

    let recipient2_balance = banks_client
        .get_account(recipient2.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient2_balance, transfer_amount2);

    // 检查手续费是否正确收取
    let bank_account_data: BankAccount = load_and_deserialize(
        banks_client.clone(),
        bank_account.pubkey(),
    )
    .await;
    assert_eq!(bank_account_data.fee, fee);
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

#[tokio::test]
async fn test_batch_transfer_sol_insufficient_balance() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额 - 故意设置较小的余额
    let initial_sender_balance = 100_000_000; // 0.1 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sender_balance,
            ..Account::default()
        },
    );

    // 为管理员账户添加余额
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
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
        &[&admin, &bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费
    let fee = 10_000_000; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();

    // 准备批量转账数据 - 故意设置大于余额的转账金额
    let transfer_amount1 = 100_000_000; // 0.1 SOL
    let transfer_amount2 = 200_000_000; // 0.2 SOL
    let transfers = vec![
        (recipient1.pubkey(), transfer_amount1),
        (recipient2.pubkey(), transfer_amount2),
    ];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    // 交易应该失败，因为余额不足
    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_batch_transfer_sol_zero_fee() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let recipient1 = Keypair::new();
    let recipient2 = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额
    let initial_sender_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sender_balance,
            ..Account::default()
        },
    );

    // 为管理员账户添加余额
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
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
        &[&admin, &bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费为 0
    let fee = 0;
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();

    // 准备批量转账数据
    let transfer_amount1 = 100_000_000; // 0.1 SOL
    let transfer_amount2 = 200_000_000; // 0.2 SOL
    let transfers = vec![
        (recipient1.pubkey(), transfer_amount1),
        (recipient2.pubkey(), transfer_amount2),
    ];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    // 添加接收者账户到 remaining_accounts
    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client
        .process_transaction(batch_transfer_tx)
        .await
        .unwrap();

    // 验证转账结果
    let total_transfer = transfer_amount1 + transfer_amount2;
    let transaction_fee = 5000; // 预估的交易费用
    let expected_sender_balance = initial_sender_balance - total_transfer - transaction_fee;

    // 检查发送者余额
    let sender_balance = banks_client
        .get_account(sender.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(sender_balance, expected_sender_balance);

    // 检查接收者余额
    let recipient1_balance = banks_client
        .get_account(recipient1.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient1_balance, transfer_amount1);

    let recipient2_balance = banks_client
        .get_account(recipient2.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient2_balance, transfer_amount2);
}

#[tokio::test]
async fn test_batch_transfer_sol_empty_transfers() {
    // 初始化测试环境
    let program_id = batch_transfer::ID;
    let mut pt = ProgramTest::new("batch_transfer", program_id, None);
    pt.set_compute_max_units(1200_000);

    // 创建测试账户
    let admin = Keypair::new();
    let sender = Keypair::new();
    let bank_account = Keypair::new();

    // 为测试账户添加初始余额
    let initial_sender_balance = 10_000_000_000; // 10 SOL
    pt.add_account(
        sender.pubkey(),
        Account {
            lamports: initial_sender_balance,
            ..Account::default()
        },
    );

    // 为管理员账户添加余额
    pt.add_account(
        admin.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化 bank_account
    let initialize_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::Initialize {
            bank_account: bank_account.pubkey(),
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
        &[&admin, &bank_account],
        recent_blockhash,
    );

    banks_client
        .process_transaction(initialize_tx)
        .await
        .unwrap();

    // 设置手续费
    let fee = 10_000_000; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: admin.pubkey(),
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&admin.pubkey()),
        &[&admin],
        recent_blockhash,
    );

    banks_client.process_transaction(set_fee_tx).await.unwrap();

    // 准备空的转账列表
    let transfers: Vec<(Pubkey, u64)> = vec![];

    // 执行批量转账
    let batch_transfer_ix = Instruction {
        program_id,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    // 交易应该失败，因为转账列表为空
    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sol_single_transfer() {
    let (mut pt, admin, sender, bank_account) = setup_test_env().await;
    let recipient = Keypair::new();
    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化并设置手续费
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 执行单笔转账
    let transfer_amount = LAMPORTS_PER_SOL; // 1 SOL
    let transfers = vec![(recipient.pubkey(), transfer_amount)];

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client
        .process_transaction(batch_transfer_tx)
        .await
        .unwrap();

    // 验证转账结果
    let recipient_balance = banks_client
        .get_account(recipient.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(recipient_balance, transfer_amount);
}

#[tokio::test]
async fn test_sol_large_amount_overflow() {
    let (mut pt, admin, sender, bank_account) = setup_test_env().await;
    let recipient = Keypair::new();
    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化并设置手续费
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 尝试转账超大金额
    let transfer_amount = u64::MAX;
    let transfers = vec![(recipient.pubkey(), transfer_amount)];

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    // 预期交易会失败
    let result = banks_client.process_transaction(batch_transfer_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_unauthorized_set_fee() {
    let (mut pt, admin, sender, bank_account) = setup_test_env().await;
    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;

    // 非管理员尝试设置手续费
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    let set_fee_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::SetFee {
            bank_account: bank_account.pubkey(),
            admin: sender.pubkey(), // 使用非管理员账户
        }
        .to_account_metas(None),
        data: batch_transfer::instruction::SetFee { fee }.data(),
    };

    let set_fee_tx = Transaction::new_signed_with_payer(
        &[set_fee_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    // 预期交易会失败
    let result = banks_client.process_transaction(set_fee_tx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sol_many_recipients() {
    let (mut pt, admin, sender, bank_account) = setup_test_env().await;
    let (mut banks_client, _payer, recent_blockhash) = pt.start().await;

    // 初始化并设置手续费
    initialize_bank_account(&mut banks_client, &admin, &bank_account, recent_blockhash).await;
    let fee = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    set_fee(&mut banks_client, &admin, &bank_account, fee, recent_blockhash).await;

    // 创建多个接收者
    let recipients: Vec<Keypair> = (0..10).map(|_| Keypair::new()).collect();
    let transfer_amount = LAMPORTS_PER_SOL / 10; // 0.1 SOL each
    let transfers: Vec<(Pubkey, u64)> = recipients
        .iter()
        .map(|recipient| (recipient.pubkey(), transfer_amount))
        .collect();

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer::ID,
        accounts: batch_transfer::accounts::BatchTransferSol {
            sender: sender.pubkey(),
            bank_account: bank_account.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(Some(true)),
        data: batch_transfer::instruction::BatchTransferSol { transfers: transfers.clone() }.data(),
    };

    let mut accounts = batch_transfer_ix.accounts;
    accounts.extend(
        transfers
            .iter()
            .map(|(pubkey, _)| AccountMeta::new(*pubkey, false)),
    );

    let batch_transfer_ix = Instruction {
        program_id: batch_transfer_ix.program_id,
        accounts,
        data: batch_transfer_ix.data,
    };

    let batch_transfer_tx = Transaction::new_signed_with_payer(
        &[batch_transfer_ix],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );

    banks_client
        .process_transaction(batch_transfer_tx)
        .await
        .unwrap();

    // 验证每个接收者的余额
    for recipient in recipients {
        let balance = banks_client
            .get_account(recipient.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports;
        assert_eq!(balance, transfer_amount);
    }
}
