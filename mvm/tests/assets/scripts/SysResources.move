script {
    use 0x1::Store;
    use 0x1::DiemBlock;
    use 0x1::DiemTimestamp;

    fun store_system_resources(addr_for_block: signer, addr_for_timestamp: signer) {
        Store::store_u64(&addr_for_block, DiemBlock::get_current_block_height());
        Store::store_u64(&addr_for_timestamp, DiemTimestamp::now_microseconds());
    }
}