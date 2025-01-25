// use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
// use solana_program::instruction::Instruction;
// use solana_program::pubkey::Pubkey;
// use solana_program::system_program;
// use solana_program_test::{BanksClient, ProgramTest};
// use solana_sdk::account::Account;
// use solana_sdk::signature::Keypair;
// use solana_sdk::signer::Signer;
// use solana_sdk::transaction::Transaction;
//
// use batch_transfer::BankAccount;
//
// #[tokio::test]
// async fn test_set_fee() {
//     let SetUpTest {
//         program_id,
//         pt,
//         deployer,
//         admin,
//         bank_account,
//     } = SetUpTest::new();
//
//     let (mut banks_client, payer, recent_blockhash) = pt.start().await;
//
//     // 调用 initialize 指令来初始化 bank_account
//     let initialize_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::Initialize {
//             bank_account: bank_account.pubkey(),
//             deployer: deployer.pubkey(),
//             system_program: system_program::ID,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::Initialize { admin: admin.pubkey() }.data(),
//     };
//
//     let initialize_tx = Transaction::new_signed_with_payer(
//         &[initialize_ix],
//         Some(&deployer.pubkey()),
//         &[&deployer, &bank_account],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(initialize_tx).await.unwrap();
//
//     // 设置 fee
//     let fee = 5;
//
//     let set_fee_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::SetFee {
//             bank_account: bank_account.pubkey(),
//             admin: admin.pubkey(),
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::SetFee { fee: fee }.data(),
//     };
//
//     let set_fee_tx = Transaction::new_signed_with_payer(
//         &[set_fee_ix],
//         Some(&admin.pubkey()),
//         &[&admin],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(set_fee_tx).await.unwrap();
//
//     // 检查 bank_account 是否正确设置了 supported_token
//     let bank_account_data: BankAccount = load_and_deserialize(
//         banks_client.clone(),
//         bank_account.pubkey(),
//     ).await;
//
//     assert_eq!(bank_account_data.fee, fee);
// }
//
// pub struct SetUpTest {
//     pub program_id: Pubkey,
//     pub pt: ProgramTest,
//     pub deployer: Keypair,
//     pub admin: Keypair,
//     pub bank_account: Keypair,
// }
//
// impl SetUpTest {
//     pub fn new() -> Self {
//         let program_id = batch_transfer::ID;
//         let mut pt: ProgramTest = ProgramTest::new("batch_transfer", program_id, None);
//         pt.set_compute_max_units(1200_000);
//
//         let mut accounts: Vec<Keypair> = Vec::new();
//
//         let deployer = Keypair::new();
//         accounts.push(deployer.insecure_clone());
//         let admin = Keypair::new();
//         accounts.push(admin.insecure_clone());
//         let bank_account = Keypair::new();
//         accounts.push(bank_account.insecure_clone());
//
//
//         for account in accounts {
//             //create a new account and fund with 1 SOL
//             pt.add_account(
//                 account.pubkey(),
//                 Account {
//                     lamports: 1_000_000_000,
//                     ..Account::default()
//                 },
//             );
//         }
//
//         Self {
//             program_id,
//             pt,
//             deployer,
//             admin,
//             bank_account,
//         }
//     }
// }
//
//
// pub async fn load_and_deserialize<T: AccountDeserialize>(
//     mut banks_client: BanksClient,
//     address: Pubkey,
// ) -> T {
//     let account = banks_client
//         .get_account(address)
//         .await
//         .unwrap()
//         .unwrap();
//
//     T::try_deserialize(&mut account.data.as_slice()).unwrap()
// }
//
