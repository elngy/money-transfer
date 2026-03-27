#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, symbol_short, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,      // Địa chỉ quản trị (Anchor/Cổng thanh toán)
    FeeRate,    // Tỷ lệ phí giao dịch (ví dụ: 1 = 0.1%)
}

#[contract]
pub struct CrossBorderPayment;

#[contractimpl]
impl CrossBorderPayment {
    
    // 1. Khởi tạo hợp đồng với địa chỉ Admin (Anchor)
    pub fn initialize(env: Env, admin: Address, fee: u32) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeRate, &fee);
    }

    // 2. Hàm chuyển tiền quốc tế (Hỗ trợ USDC hoặc bất kỳ Token nào trên Stellar)
    pub fn transfer_with_compliance(
        env: Env,
        from: Address,
        to: Address,
        token_address: Address,
        amount: i128,
    ) {
        // Xác thực người gửi
        from.require_auth();

        // Bước kiểm tra tuân thủ (Mock Compliance Check)
        // Trong thực tế, AI của bạn sẽ gửi tín hiệu phê duyệt vào đây
        Self::check_compliance(&env, &from, amount);

        // Khởi tạo client cho Token (ví dụ USDC)
        let client = token::Client::new(&env, &token_address);
        let fee_rate = env.storage().instance().get::<_, u32>(&DataKey::FeeRate).unwrap_or(0);
        
        // Tính toán phí (ví dụ: amount * fee_rate / 1000)
        let fee_amount = (amount * fee_rate as i128) / 1000;
        let final_amount = amount - fee_amount;

        // Thực hiện chuyển tiền
        // Gửi phí về cho Admin (Anchor)
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if fee_amount > 0 {
            client.transfer(&from, &admin, &fee_amount);
        }

        // Gửi số tiền còn lại cho người nhận
        client.transfer(&from, &to, &final_amount);

        // Bắn sự kiện (Event) để App phía Frontend nhận được
        env.events().publish((symbol_short!("pay_out"), from), final_amount);
    }

    // 3. Hàm kiểm tra tuân thủ (Đây là nơi AI/Business Logic can thiệp)
    fn check_compliance(env: &Env, user: &Address, amount: i128) {
        // Ví dụ: Giới hạn chuyển tiền tối đa mỗi lần để chống rửa tiền (AML)
        let max_limit: i128 = 10_000_000_000; // Giả sử đơn vị là stroops
        if amount > max_limit {
            panic!("Transaction exceeds compliance limit. Manual review required.");
        }
    }

    // Hàm cập nhật phí (Chỉ Admin mới có quyền)
    pub fn set_fee(env: Env, new_fee: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::FeeRate, &new_fee);
    }
}
