use bornomala_engine::{InputScheme, KeyEvent};
use bornomala_tsf::{
    com::{DllCanUnloadNow, DllGetClassObject, DllRegisterServer, DllUnregisterServer, S_OK},
    BornomalaTextService, SimulatedHostAdapter,
};

#[test]
fn test_simulated_host_basic_typing() {
    let mut service = BornomalaTextService::new();
    let mut host = SimulatedHostAdapter::new();

    // Classic Unicode: 'k' (KeyJ in fixed layout, key_code 74 = 'J') -> 'ক'
    let event_k = KeyEvent {
        key_code: 74,
        ch: Some('j'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    let consumed = service.handle_key(event_k, &mut host);
    assert!(consumed);
    assert_eq!(host.committed_buffer, "ক");

    // Classic Unicode: 'g' (hasanta, key_code 71 = 'G'), 'j' -> 'ক্ক'
    let event_g = KeyEvent {
        key_code: 71,
        ch: Some('g'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    let consumed = service.handle_key(event_g, &mut host);
    assert!(consumed);
    assert_eq!(host.committed_buffer, "ক্");

    let event_k2 = KeyEvent {
        key_code: 74,
        ch: Some('j'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    let consumed = service.handle_key(event_k2, &mut host);
    assert!(consumed);
    assert_eq!(host.committed_buffer, "ক্ক");
}

#[test]
fn test_phonetic_typing_and_commit() {
    let mut service = BornomalaTextService::new();
    service.set_scheme(InputScheme::Phonetic);
    let mut host = SimulatedHostAdapter::new();

    // 'a' -> "আ" in composition
    let event_a = KeyEvent {
        key_code: 65,
        ch: Some('a'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    service.handle_key(event_a, &mut host);
    assert_eq!(host.composition_buffer.as_deref(), Some("আ"));

    // 'm' -> "আম" in composition
    let event_m = KeyEvent {
        key_code: 77,
        ch: Some('m'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    service.handle_key(event_m, &mut host);
    assert_eq!(host.composition_buffer.as_deref(), Some("আম"));

    // ' ' (Space, key_code 32) commits composition
    let event_space = KeyEvent {
        key_code: 32,
        ch: Some(' '),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    service.handle_key(event_space, &mut host);
    assert_eq!(host.committed_buffer, "আম ");
    assert_eq!(host.composition_buffer, None);
}

#[test]
fn test_watchdog_fail_safe_pass_through() {
    let mut service = BornomalaTextService::new();
    let mut host = SimulatedHostAdapter::new();

    assert!(!service.is_watchdog_tripped());

    // Record failures up to threshold
    for i in 1..=4 {
        service.record_failure(i);
        assert!(!service.is_watchdog_tripped());
    }

    // 5th failure trips watchdog
    service.record_failure(5);
    assert!(service.is_watchdog_tripped());

    // Now all keys should pass through without being consumed or modifying buffer
    let event_k = KeyEvent {
        key_code: 74,
        ch: Some('j'),
        shift: false,
        ctrl: false,
        alt: false,
        is_down: true,
    };
    let consumed = service.handle_key(event_k, &mut host);
    assert!(!consumed);
    assert_eq!(host.committed_buffer, "");
}

#[test]
fn test_com_entry_points_panic_safety() {
    unsafe {
        assert_eq!(DllCanUnloadNow(), S_OK);
        assert_eq!(DllRegisterServer(), S_OK);
        assert_eq!(DllUnregisterServer(), S_OK);

        let mut ppv = std::ptr::null_mut();
        let res = DllGetClassObject(std::ptr::null(), std::ptr::null(), &mut ppv);
        assert_eq!(res, S_OK);
    }
}
