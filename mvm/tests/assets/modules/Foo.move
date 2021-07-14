address 0x1 {
    module Foo {
        use 0x1::Store;

        public fun foo(account: &signer, val: u64) {
            Store::store_u64(account, val);
        }
    }
}