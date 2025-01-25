// use anchor_lang::prelude::*;
// use anchor_lang::InstructionData;
// use solana_program::instruction::Instruction;
// use solana_program::system_program;
// use solana_program_test::*;
// use solana_sdk::account::Account;
// use solana_sdk::signature::Keypair;
// use solana_sdk::{
//     signature::Signer,
//     transaction::Transaction,
// };
//
// use batch_transfer::{self, UserAccount};
//
// #[tokio::test]
// async fn test_reset_user_account() {
//     let SetUpTest {
//         program_id,
//         pt,
//         admin,
//         user_account,
//     } = SetUpTest::new();
//
//
//     let (mut banks_client, payer, recent_blockhash) = pt.start().await;
//
//     let reset_user_account_ix = Instruction {
//         program_id: program_id,
//         accounts: batch_transfer::accounts::ResetUserAccount {
//             user_account: user_account.pubkey(),
//             admin: admin.pubkey(),
//             system_program: system_program::ID,
//         }
//             .to_account_metas(None),
//         data: batch_transfer::instruction::ResetUserAccount {}.data(),
//     };
//
//     let reset_user_account_tx = Transaction::new_signed_with_payer(
//         &[reset_user_account_ix],
//         Some(&admin.pubkey()),
//         &[&admin, &user_account],
//         recent_blockhash,
//     );
//
//     banks_client.process_transaction(reset_user_account_tx).await.unwrap();
//
//     // 反序列化并检查用户账户
//     let user_account_data: UserAccount = load_and_deserialize(
//         banks_client.clone(),
//         user_account.pubkey(),
//     ).await;
//
//     // 确保用户账户的余额已更新
//     assert_eq!(user_account_data.amount, 0);
// }
//
//
// pub struct SetUpTest {
//     pub program_id: Pubkey,
//     pub pt: ProgramTest,
//     pub admin: Keypair,
//     pub user_account: Keypair,
// }
//
// impl SetUpTest {
//     pub fn new() -> Self {
//         let program_id = batch_transfer::ID;
//         let mut pt: ProgramTest = ProgramTest::new("batch_transfer", program_id, None);
//         pt.set_compute_max_units(1200_000);
//
//         let mut accounts: Vec<Keypair> = Vec::new();
//         let admin = Keypair::new();
//         accounts.push(admin.insecure_clone());
//         let user_account = Keypair::new();
//         accounts.push(user_account.insecure_clone());
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
//             admin,
//             user_account,
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
//
//
