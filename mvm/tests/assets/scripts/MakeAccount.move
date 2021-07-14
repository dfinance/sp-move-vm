script {
    use 0x1::DiemAccount;
    use 0x1::PONT::PONT;

    fun make_account(root: signer, addr: address) {
        DiemAccount::create_child_vasp_account<PONT>(&root, addr, x"", true);
    }
}