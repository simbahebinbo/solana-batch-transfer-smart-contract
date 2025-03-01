use solana_program::program_error::ProgramError;
use batch_transfer::{TransferInfo, safe_add, safe_sum_transfer_info};
use solana_program::pubkey::Pubkey;

#[test]
fn test_safe_add() {
    // 正常情况
    let result = safe_add(10, 20);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 30);

    // 溢出情况
    let result = safe_add(u64::MAX, 1);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProgramError::InvalidArgument);
}

#[test]
fn test_safe_sum_transfer_info() {
    // 创建测试数据
    let transfer1 = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: 100,
    };
    let transfer2 = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: 200,
    };
    let transfers = vec![transfer1.clone(), transfer2]; // 使用 clone 避免所有权移动

    // 测试正常情况
    let result = safe_sum_transfer_info(&transfers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 300);

    // 创建可能导致溢出的测试数据
    let transfer_overflow = TransferInfo {
        recipient: Pubkey::new_unique(),
        amount: u64::MAX,
    };
    let transfers_overflow = vec![transfer1, transfer_overflow];

    // 测试溢出情况
    let result = safe_sum_transfer_info(&transfers_overflow);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ProgramError::InvalidArgument);
} 