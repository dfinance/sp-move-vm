module Store {
    struct U64 has copy, key, store, drop { val: u64 }

    struct U128 has copy, key, store, drop { val: u128 }

    struct Address has copy, key, store, drop { val: address }

    struct VectorU8 has copy, key, store, drop { val: vector<u8> }

    struct Res<R: store> has store, key { val: R }

    public fun store_u64(account: &signer, val: u64) {
        let foo = U64 { val: val };
        move_to<U64>(account, foo);
    }

    public fun store_u128(account: &signer, val: u128) {
        let foo = U128 { val: val };
        move_to<U128>(account, foo);
    }

    public fun store_address(account: &signer, val: address) {
        let addr = Address { val: val };
        move_to<Address>(account, addr);
    }

    public fun store_vector_u8(account: &signer, val: vector<u8>) {
        let vec = VectorU8 { val: val };
        move_to<VectorU8>(account, vec);
    }

    public fun store_res<R: store>(account: &signer, val: R) {
        let res = Res { val: val };
        move_to<Res<R>>(account, res);
    }
}