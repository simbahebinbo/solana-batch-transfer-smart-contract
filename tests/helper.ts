import { Keypair, LAMPORTS_PER_SOL, PublicKey, Connection } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";

/**
 * 等待一段时间（用于等待交易确认）
 * @param ms 等待的毫秒数
 */
export const sleep = (ms: number): Promise<void> => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

/**
 * 向指定账户请求SOL空投
 * @param connection Solana连接实例
 * @param recipient 接收SOL的账户公钥 
 * @param amount 请求的SOL数量
 */
export const requestAirdrop = async (
  connection: Connection,
  recipient: PublicKey,
  amount: number = 10 * LAMPORTS_PER_SOL
): Promise<void> => {
  const signature = await connection.requestAirdrop(recipient, amount);
  await connection.confirmTransaction(signature);
};

/**
 * 创建测试用SPL代币
 * @param connection Solana连接实例
 * @param payer 支付者账户
 * @param mintAuthority 铸币权限账户
 * @param decimals 小数位数
 */
export const createTestToken = async (
  connection: Connection,
  payer: Keypair,
  mintAuthority: PublicKey = payer.publicKey,
  decimals: number = 9
): Promise<PublicKey> => {
  return await createMint(
    connection,
    payer,
    mintAuthority,
    null, // 冻结权限（null表示没有）
    decimals
  );
};

/**
 * 创建或获取关联代币账户并返回地址
 * @param connection Solana连接实例
 * @param payer 支付者账户
 * @param mint 代币铸币账户
 * @param owner 代币所有者
 */
export const getTestTokenAccount = async (
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> => {
  const account = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    owner
  );
  return account.address;
};

/**
 * 为测试账户铸造一定数量的代币
 * @param connection Solana连接实例
 * @param payer 支付者账户
 * @param mint 代币铸币账户
 * @param destination 目标代币账户
 * @param authority 铸币权限账户
 * @param amount 铸造金额
 */
export const mintTestTokens = async (
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number = 1000 * LAMPORTS_PER_SOL
): Promise<void> => {
  await mintTo(
    connection,
    payer,
    mint,
    destination,
    authority.publicKey,
    amount
  );
};

/**
 * 初始化一组测试账户并为其提供SOL
 * @param connection Solana连接实例
 * @param count 需要创建的账户数量
 * @param solAmount 每个账户获得的SOL数量
 */
export const initializeTestAccounts = async (
  connection: Connection,
  count: number = 5,
  solAmount: number = 10 * LAMPORTS_PER_SOL
): Promise<Keypair[]> => {
  const accounts = Array(count).fill(0).map(() => Keypair.generate());
  
  // 为每个账户提供SOL
  const promises = accounts.map(account => 
    requestAirdrop(connection, account.publicKey, solAmount)
  );
  
  await Promise.all(promises);
  return accounts;
}; 