script {
    use 0x1::DiemAccount;
    use 0x1::PONT::PONT;
    use 0x1::Signer;
    use 0x1::Errors;

    fun transfer(from: signer, to: signer, from_balance: u64, to_balance: u64, to_move: u64) {
        assert(DiemAccount::balance<PONT>(Signer::address_of(&from)) == from_balance, Errors::custom(0));
        assert(DiemAccount::balance<PONT>(Signer::address_of(&to)) == to_balance, Errors::custom(1));

        let alise_withdraw_cap = DiemAccount::extract_withdraw_capability(&from);
        DiemAccount::pay_from<PONT>(&alise_withdraw_cap, Signer::address_of(&to), to_move, x"", x"");
        DiemAccount::restore_withdraw_capability(alise_withdraw_cap);

        assert(DiemAccount::balance<PONT>(Signer::address_of(&from)) == from_balance - to_move, Errors::custom(2));
        assert(DiemAccount::balance<PONT>(Signer::address_of(&to)) == to_balance + to_move, Errors::custom(3));
    }
}