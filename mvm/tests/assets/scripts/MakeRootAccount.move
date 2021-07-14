script {
    use 0x1::DiemAccount;
    use 0x1::PONT::PONT;

    fun make_root_account(root: signer, addr: address) {
        DiemAccount::create_parent_vasp_account<PONT>(&root, addr, x"", b"VASP_FATHER", true);
    }
}