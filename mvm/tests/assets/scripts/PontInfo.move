script {
    use 0x1::Diem;
    use 0x1::PONT::PONT;
    fun pont_info(expected: u128) {
        assert(expected == Diem::market_cap<PONT>(), 1);
    }
}