script {
    use 0x1::Abort;

    fun error(_signer: signer) {
        Abort::error(13);
    }
}