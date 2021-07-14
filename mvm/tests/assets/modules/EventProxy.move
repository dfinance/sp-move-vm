module EventProxy {
    use 0x1::Event;

    struct U64 has copy, key, store, drop { val: u64 }

    public fun emit_event(addr: &signer, val: u64) {
        Event::publish_generator(addr);
        let handle = Event::new_event_handle<U64>(addr);
        Event::emit_event(&mut handle, U64 { val });
        Event::destroy_handle(handle);
    }

    public fun create_val(val: u64): U64 {
        U64 { val }
    }
}